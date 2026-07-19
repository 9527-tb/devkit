//! 进程宿主：启动 / 停止 / 日志缓冲 / 端口探测。
//!
//! 对应 DESIGN.md §12.3。
//! DONE(rs-instance-key): 进程用 projectKey；日志按项目追加（path::kind）— DESIGN §16.1

pub mod summary;

use crate::core::project_ref::make_project_key;
use crate::core::scan_engine::{project_at, resolve_command};
use crate::models::{LogEvent, ProcessView, RunSummary};
use crate::runtime::{self, RuntimeEntry, RuntimeSettings};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use tauri::{AppHandle, Emitter};

/// 项目日志键：同一 path+kind 下命令日志追加，不因进程结束/换 pid 清空。
fn make_log_key(path: &str, kind: &str) -> String {
    make_project_key(&normalize_path(path), kind)
}

pub struct Running {
    pub child: Child,
    pub action: String,
    pub port: Option<String>,
    pub kind: String,
    pub started_at: Instant,
}

pub struct AppState {
    pub processes: Arc<Mutex<HashMap<String, Vec<Running>>>>,
    pub logs: Arc<Mutex<HashMap<String, Vec<String>>>>,
    /// 每个 projectKey 最近一次运行摘要
    pub last_run_summaries: Arc<Mutex<HashMap<String, RunSummary>>>,
    /// 每个 projectKey 最近观测到的端口（供体检弱检查）
    pub last_ports: Arc<Mutex<HashMap<String, String>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            logs: Arc::new(Mutex::new(HashMap::new())),
            last_run_summaries: Arc::new(Mutex::new(HashMap::new())),
            last_ports: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// 具体监听端口（1–65535）。`0` 表示随机分配，启动前不可用。
pub fn concrete_listen_port(raw: &str) -> Option<u16> {
    let n: u16 = raw.trim().parse().ok()?;
    if n == 0 {
        None
    } else {
        Some(n)
    }
}

/// 仅从明确的端口参数推断；禁止把 Maven `-p profile` 当成端口。
/// `port=0`（随机端口）视为未指定。
fn port_from_command(command: &str) -> Option<String> {
    let parts: Vec<_> = command.split_whitespace().collect();
    for (i, p) in parts.iter().enumerate() {
        if *p == "--port" {
            if let Some(v) = parts.get(i + 1) {
                if v.chars().all(|c| c.is_ascii_digit()) {
                    return concrete_listen_port(v).map(|n| n.to_string());
                }
            }
        } else if let Some(rest) = p.strip_prefix("--port=") {
            if rest.chars().all(|c| c.is_ascii_digit()) {
                return concrete_listen_port(rest).map(|n| n.to_string());
            }
        } else if let Some(rest) = p.strip_prefix("-Dserver.port=") {
            if rest.chars().all(|c| c.is_ascii_digit()) {
                return concrete_listen_port(rest).map(|n| n.to_string());
            }
        } else if let Some(rest) = p.strip_prefix("PORT=") {
            if rest.chars().all(|c| c.is_ascii_digit()) {
                return concrete_listen_port(rest).map(|n| n.to_string());
            }
        }
    }
    None
}

pub fn parse_port_from_line(line: &str) -> Option<String> {
    let patterns = [
        r"https?://(?:localhost|127\.0\.0\.1|0\.0\.0\.0):(\d+)",
        r"Local:\s+https?://[^:]+:(\d+)",
        r"Tomcat started on port(?:\(s\))?\s*:\s*(\d+)",
        r"listening on (?:port )?(\d+)",
        r"started server on [^:]*:(\d+)",
    ];
    for pat in patterns {
        if let Ok(re) = Regex::new(pat) {
            if let Some(caps) = re.captures(line) {
                if let Some(m) = caps.get(1) {
                    return concrete_listen_port(m.as_str()).map(|n| n.to_string());
                }
            }
        }
    }
    None
}

pub fn port_for_pid(pid: u32) -> Option<String> {
    crate::platform::list_listening_ports_tree(pid)
        .into_iter()
        .next()
        .map(|p| p.to_string())
}

fn normalize_path(path: &str) -> String {
    path.trim_end_matches('/').to_string()
}

fn strip_ansi(input: &str) -> String {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"\x1B\[[0-9;?]*[ -/]*[@-~]|\x1B[@-Z\\-_]|\r").expect("ansi regex")
    });
    re.replace_all(input, "").into_owned()
}

/// Keep SGR color sequences for the UI; drop carriage returns and non-color escapes.
fn sanitize_log_line(input: &str) -> String {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = RE.get_or_init(|| {
        // Pattern uses regex escapes (\\x1B / \\r), not literal control chars.
        Regex::new(r"\r|\x1B\][^\x07\x1B]*(?:\x07|\x1B\\)|\x1B\[[0-9;?]*[ -/]*[@-ln-zA-Z]|\x1B[@-Z\\-_]")
            .expect("sanitize regex")
    });
    re.replace_all(input, "").into_owned()
}

fn append_log(
    logs: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    app: &AppHandle,
    path: &str,
    kind: &str,
    pid: u32,
    line: String,
) {
    let path = normalize_path(path);
    let log_key = make_log_key(&path, kind);
    let line = sanitize_log_line(&line);
    {
        let mut store = logs.lock().unwrap();
        let entries = store.entry(log_key).or_default();
        entries.push(line.clone());
        if entries.len() > 3000 {
            entries.drain(0..800);
        }
    }
    let _ = app.emit(
        "project-log",
        LogEvent {
            path,
            kind: kind.to_string(),
            pid,
            line,
        },
    );
}

fn update_port_from_line(
    processes: &Arc<Mutex<HashMap<String, Vec<Running>>>>,
    last_ports: &Arc<Mutex<HashMap<String, String>>>,
    project_key: &str,
    pid: u32,
    line: &str,
) {
    if let Some(port) = parse_port_from_line(line) {
        // 仅记录启动后解析到的具体端口；0/随机端口不写入「常用端口」
        if let Ok(mut map) = processes.lock() {
            if let Some(items) = map.get_mut(project_key) {
                if let Some(item) = items.iter_mut().find(|r| r.child.id() == pid) {
                    item.port = Some(port.clone());
                }
            }
        }
        if let Ok(mut ports) = last_ports.lock() {
            ports.insert(project_key.to_string(), port);
        }
    }
}

fn emit_line(
    logs: &Arc<Mutex<HashMap<String, Vec<String>>>>,
    app: &AppHandle,
    processes: &Arc<Mutex<HashMap<String, Vec<Running>>>>,
    last_ports: &Arc<Mutex<HashMap<String, String>>>,
    path: &str,
    kind: &str,
    project_key: &str,
    pid: u32,
    line: String,
) {
    update_port_from_line(processes, last_ports, project_key, pid, &strip_ansi(&line));
    append_log(logs, app, path, kind, pid, line);
}

/// Read process output by bytes so Maven progress (`\r`) and buffered chunks still surface.
fn spawn_log_reader<R: Read + Send + 'static>(
    stream: R,
    logs: Arc<Mutex<HashMap<String, Vec<String>>>>,
    app: AppHandle,
    path: String,
    kind: String,
    project_key: String,
    processes: Arc<Mutex<HashMap<String, Vec<Running>>>>,
    last_ports: Arc<Mutex<HashMap<String, String>>>,
    pid: u32,
) {
    thread::spawn(move || {
        let mut stream = stream;
        let mut chunk = [0u8; 4096];
        let mut buf: Vec<u8> = Vec::new();
        loop {
            match stream.read(&mut chunk) {
                Ok(0) => {
                    if !buf.is_empty() {
                        let line = String::from_utf8_lossy(&buf).into_owned();
                        buf.clear();
                        emit_line(
                            &logs,
                            &app,
                            &processes,
                            &last_ports,
                            &path,
                            &kind,
                            &project_key,
                            pid,
                            line,
                        );
                    }
                    break;
                }
                Ok(n) => {
                    for &b in &chunk[..n] {
                        if b == b'\n' || b == b'\r' {
                            if !buf.is_empty() {
                                let line = String::from_utf8_lossy(&buf).into_owned();
                                buf.clear();
                                emit_line(
                                    &logs,
                                    &app,
                                    &processes,
                                    &last_ports,
                                    &path,
                                    &kind,
                                    &project_key,
                                    pid,
                                    line,
                                );
                            }
                        } else {
                            buf.push(b);
                            // Flush very long lines without newline (download bars, etc.)
                            if buf.len() >= 2048 {
                                let line = String::from_utf8_lossy(&buf).into_owned();
                                buf.clear();
                                emit_line(
                                    &logs,
                                    &app,
                                    &processes,
                                    &last_ports,
                                    &path,
                                    &kind,
                                    &project_key,
                                    pid,
                                    line,
                                );
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });
}

pub fn kill_process_tree(pid: u32) {
    if crate::platform::kill_tree(pid).is_ok() {
        return;
    }
    // 回退：直接 kill 单进程
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-KILL", &pid.to_string()])
            .status();
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status();
    }
    let _ = pid;
}


fn append_env_flag(current: &mut String, flag: &str) {
    if current.contains(flag) {
        return;
    }
    if !current.is_empty() {
        current.push(' ');
    }
    current.push_str(flag);
}

fn path_separator() -> &'static str {
    #[cfg(windows)]
    {
        ";"
    }
    #[cfg(not(windows))]
    {
        ":"
    }
}

fn prepend_path(cmd: &mut Command, bin_dir: &Path) {
    // GUI 应用进程 PATH 往往不含 Homebrew/nvm，子进程需用补全后的 PATH
    let path = crate::platform::enriched_path();
    let sep = path_separator();
    let prefix = bin_dir.display().to_string();
    let merged = if path.is_empty() {
        prefix
    } else {
        format!("{prefix}{sep}{path}")
    };
    cmd.env("PATH", merged);
}

fn apply_enriched_path(cmd: &mut Command) {
    cmd.env("PATH", crate::platform::enriched_path());
}

fn node_bin_dirs(entry: &RuntimeEntry) -> Vec<PathBuf> {
    let home = PathBuf::from(&entry.path);
    let bin = if home.join("bin/node").is_file() {
        home.join("bin")
    } else if home.file_name().and_then(|n| n.to_str()) == Some("bin") {
        home.clone()
    } else if home.join("node").is_file() {
        home.parent().unwrap_or(&home).to_path_buf()
    } else {
        home.join("bin")
    };
    vec![bin]
}

/// 将 program 解析为绝对路径。Command::new("yarn") 使用的是当前进程 PATH，不是子进程 env。
fn resolve_program_path(
    program: &str,
    kind: &str,
    settings: &RuntimeSettings,
    project_path: &Path,
) -> Result<String, String> {
    if Path::new(program).is_absolute() {
        return Ok(program.to_string());
    }

    let mut extra: Vec<PathBuf> = Vec::new();
    if kind == "node" {
        if let Ok((Some(_), Some(entry))) = runtime::resolve_node(settings, project_path) {
            extra.extend(node_bin_dirs(&entry));
        }
        // 未锁定版本时，也把已配置的 Node bin 放进搜索路径
        for entry in &settings.node.nodes {
            extra.extend(node_bin_dirs(entry));
        }
    }

    crate::platform::find_executable_with_bins(program, &extra)
        .map(|p| p.to_string_lossy().into_owned())
        .ok_or_else(|| {
            format!(
                "未找到可执行文件 `{program}`。\n请确认已安装并在终端可用，或在「设置」中配置 Node / JDK。"
            )
        })
}

fn apply_jdk_env(cmd: &mut Command, entry: &RuntimeEntry, version: &str) -> String {
    let java_home = PathBuf::from(&entry.path);
    cmd.env("JAVA_HOME", &java_home);
    prepend_path(cmd, &java_home.join("bin"));
    format!("{} (Java {})", java_home.display(), version)
}

fn apply_node_env(cmd: &mut Command, entry: &RuntimeEntry, version: &str) -> String {
    let home = PathBuf::from(&entry.path);
    let bin = if home.join("bin/node").is_file() {
        home.join("bin")
    } else if home.file_name().and_then(|n| n.to_str()) == Some("bin") {
        home.clone()
    } else if home.join("node").is_file() {
        home.parent().unwrap_or(&home).to_path_buf()
    } else {
        home.join("bin")
    };
    prepend_path(cmd, &bin);
    format!("{} (Node {})", home.display(), version)
}

fn apply_maven_tool_opts(cmd: &mut Command) {
    let mut tool_opts = std::env::var("JAVA_TOOL_OPTIONS").unwrap_or_default();
    append_env_flag(&mut tool_opts, "-Dfile.encoding=UTF-8");
    // Helps Dubbo 3.3.x skip broken optional JSON providers during ServiceLoader scan.
    append_env_flag(&mut tool_opts, "-Ddubbo.json-framework.prefer=fastjson2");
    cmd.env("JAVA_TOOL_OPTIONS", &tool_opts);

    let mut maven_opts = std::env::var("MAVEN_OPTS").unwrap_or_default();
    append_env_flag(&mut maven_opts, "-Djansi.passthrough=true");
    cmd.env("MAVEN_OPTS", &maven_opts);
}

fn resolve_runtime_notes(
    app: &AppHandle,
    cmd: &mut Command,
    project_path: &str,
    kind: &str,
) -> Result<Vec<String>, String> {
    let settings: RuntimeSettings = runtime::load_settings(app)?;
    let path = Path::new(project_path);
    let mut notes = Vec::new();

    if kind == "maven" {
        apply_maven_tool_opts(cmd);
        let (version, entry) = runtime::resolve_jdk(&settings, path)?;
        if let (Some(ver), Some(e)) = (version, entry) {
            notes.push(format!("[DevKit] JAVA_HOME={}", apply_jdk_env(cmd, &e, &ver)));
        }
    } else if kind == "gradle" {
        let (version, entry) = runtime::resolve_jdk(&settings, path)?;
        if let (Some(ver), Some(e)) = (version, entry) {
            notes.push(format!("[DevKit] JAVA_HOME={}", apply_jdk_env(cmd, &e, &ver)));
        }
    } else if kind == "node" {
        let mut tool_opts = std::env::var("JAVA_TOOL_OPTIONS").unwrap_or_default();
        append_env_flag(&mut tool_opts, "-Dfile.encoding=UTF-8");
        cmd.env("JAVA_TOOL_OPTIONS", tool_opts);

        let (version, entry) = runtime::resolve_node(&settings, path)?;
        if let (Some(ver), Some(e)) = (version, entry) {
            notes.push(format!("[DevKit] NODE={}", apply_node_env(cmd, &e, &ver)));
        }
    }

    Ok(notes)
}

pub fn run_action(
    app: AppHandle,
    state: &AppState,
    path: String,
    action: String,
    kind: String,
) -> Result<ProcessView, String> {
    let project = project_at(std::path::Path::new(&path), &kind).ok_or("无法识别项目")?;
    let (mut program, args) = resolve_command(&project, &action)?;
    // 过渡：日志仍按 path 键；同时用 projectKey 写入以便 R3 双读
    let _project_key = make_project_key(&path, &kind);
    let settings = runtime::load_settings(&app)?;
    if project.kind == "maven" {
        program = runtime::maven_bin(&settings);
    } else if project.kind == "node" {
        // 以设置中的包管理器为准（默认 npm），不再按 lockfile 自动切换
        program = runtime::normalize_node_package_manager(&settings.node.package_manager).into();
    } else if project.kind == "gradle" {
        if let Some(wrapper) =
            crate::providers::gradle::actions::find_gradle_wrapper(Path::new(&path))
        {
            program = wrapper.to_string_lossy().into();
        }
    }
    // 必须解析为绝对路径：Unix 上 Command::new("yarn") 只用当前进程 PATH，GUI 下常找不到
    program = resolve_program_path(&program, &project.kind, &settings, Path::new(&path))?;

    let display_cmd = {
        let base = Path::new(&program)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&program);
        if args.is_empty() {
            base.to_string()
        } else {
            format!("{base} {}", args.join(" "))
        }
    };
    let mut port = port_from_command(&display_cmd);

    // Spawn program directly (no login shell) so stdout/stderr are reliably piped.
    let mut cmd = Command::new(&program);
    cmd.args(&args)
        .current_dir(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());
    apply_enriched_path(&mut cmd);

    let runtime_notes = resolve_runtime_notes(&app, &mut cmd, &path, &project.kind)?;

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                if libc::setpgid(0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    let mut child = cmd.spawn().map_err(|e| format!("启动失败 ({program}): {e}"))?;
    let pid = child.id();
    if port.is_none() {
        port = port_for_pid(pid);
    }

    let project_key = make_project_key(&path, &kind);
    let logs = state.logs.clone();
    let processes = state.processes.clone();
    let last_ports = state.last_ports.clone();
    if let Some(ref p) = port {
        if concrete_listen_port(p).is_some() {
            if let Ok(mut ports) = last_ports.lock() {
                ports.insert(project_key.clone(), p.clone());
            }
        }
    }
    // 每次执行前追加分割线，日志跨命令保留
    append_log(
        &logs,
        &app,
        &path,
        &kind,
        pid,
        "────────────────────────────────────────".into(),
    );
    append_log(
        &logs,
        &app,
        &path,
        &kind,
        pid,
        format!("[DevKit] 执行命令: {}", display_cmd),
    );
    for note in runtime_notes {
        append_log(&logs, &app, &path, &kind, pid, note);
    }

    if let Some(stream) = child.stdout.take() {
        spawn_log_reader(
            stream,
            logs.clone(),
            app.clone(),
            path.clone(),
            kind.clone(),
            project_key.clone(),
            processes.clone(),
            last_ports.clone(),
            pid,
        );
    }
    if let Some(stream) = child.stderr.take() {
        spawn_log_reader(
            stream,
            logs.clone(),
            app.clone(),
            path.clone(),
            kind.clone(),
            project_key.clone(),
            processes.clone(),
            last_ports.clone(),
            pid,
        );
    }

    let view = ProcessView {
        pid,
        action: action.clone(),
        port: port.clone(),
        kind: kind.clone(),
    };
    let started_at = Instant::now();
    state
        .processes
        .lock()
        .map_err(|_| "进程状态不可用")?
        .entry(project_key.clone())
        .or_default()
        .push(Running {
            child,
            action: view.action.clone(),
            port: view.port.clone(),
            kind: kind.clone(),
            started_at,
        });

    // Ensure the frontend sees a completion marker even if the process finishes quickly.
    let logs_done = state.logs.clone();
    let procs_done = state.processes.clone();
    let summaries_done = state.last_run_summaries.clone();
    let ports_done = state.last_ports.clone();
    let app_done = app.clone();
    let path_done = project_key.clone();
    let log_path = path.clone();
    let log_kind = kind.clone();
    let action_done = action.clone();
    thread::spawn(move || {
        // Poll until this pid disappears or exits.
        loop {
            thread::sleep(std::time::Duration::from_millis(400));
            let mut finished: Option<(Option<i32>, u64, Option<String>)> = None;
            if let Ok(mut map) = procs_done.lock() {
                if let Some(items) = map.get_mut(&path_done) {
                    if let Some(pos) = items.iter_mut().position(|r| r.child.id() == pid) {
                        match items[pos].child.try_wait() {
                            Ok(Some(status)) => {
                                let code = status.code();
                                let duration_ms = items[pos].started_at.elapsed().as_millis() as u64;
                                let port = items[pos].port.clone();
                                items.remove(pos);
                                if items.is_empty() {
                                    map.remove(&path_done);
                                }
                                finished = Some((code, duration_ms, port));
                            }
                            Ok(None) => {}
                            Err(_) => {
                                let duration_ms = items[pos].started_at.elapsed().as_millis() as u64;
                                let port = items[pos].port.clone();
                                items.remove(pos);
                                if items.is_empty() {
                                    map.remove(&path_done);
                                }
                                finished = Some((None, duration_ms, port));
                            }
                        }
                    } else {
                        // Already reaped elsewhere.
                        break;
                    }
                } else {
                    break;
                }
            }
            if let Some((code, duration_ms, port)) = finished {
                let msg = match code {
                    Some(0) => "[DevKit] 进程已结束 (exit 0)".to_string(),
                    Some(c) => format!("[DevKit] 进程已结束 (exit {c})"),
                    None => "[DevKit] 进程已结束".into(),
                };
                append_log(&logs_done, &app_done, &log_path, &log_kind, pid, msg);

                let log_lines = {
                    let key = make_log_key(&log_path, &log_kind);
                    logs_done
                        .lock()
                        .ok()
                        .and_then(|m| m.get(&key).cloned())
                        .unwrap_or_default()
                };
                let summary = summary::build_summary(
                    path_done.clone(),
                    normalize_path(&log_path),
                    log_kind.clone(),
                    pid,
                    action_done.clone(),
                    code,
                    duration_ms,
                    port.clone(),
                    &log_lines,
                );
                if let Some(ref p) = port {
                    if let Ok(mut ports) = ports_done.lock() {
                        ports.insert(path_done.clone(), p.clone());
                    }
                }
                if let Ok(mut map) = summaries_done.lock() {
                    map.insert(path_done.clone(), summary.clone());
                }
                let _ = app_done.emit("run-finished", summary);
                break;
            }
        }
    });

    Ok(view)
}

fn process_keys_for(path: &str, kind: &str) -> Vec<String> {
    let pk = make_project_key(path, kind);
    let path = normalize_path(path);
    if pk == path {
        vec![pk]
    } else {
        vec![pk, path]
    }
}

pub fn stop_project(
    app: &AppHandle,
    state: &AppState,
    path: String,
    kind: String,
) -> Result<(), String> {
    let keys = process_keys_for(&path, &kind);
    let mut map = state.processes.lock().map_err(|_| "进程状态不可用")?;
    let mut stopped_pids = Vec::new();
    for key in keys {
        let Some(items) = map.get_mut(&key) else {
            continue;
        };
        let mut kept = Vec::new();
        for mut item in items.drain(..) {
            // projectKey 桶内均为同 kind；旧 path 桶按 kind 过滤
            if key.contains("::") || item.kind == kind {
                let pid = item.child.id();
                let _ = item.child.kill();
                kill_process_tree(pid);
                stopped_pids.push(pid);
            } else {
                kept.push(item);
            }
        }
        *items = kept;
        if items.is_empty() {
            map.remove(&key);
        }
    }
    drop(map);
    for pid in stopped_pids {
        append_log(
            &state.logs,
            app,
            &path,
            &kind,
            pid,
            "[DevKit] 已发送停止信号".into(),
        );
    }
    Ok(())
}

/// 停止所有托管中的运行实例，返回停止数量。
pub fn stop_all(app: &AppHandle, state: &AppState) -> Result<u32, String> {
    let mut map = state.processes.lock().map_err(|_| "进程状态不可用")?;
    let mut stopped_meta: Vec<(String, String, u32)> = Vec::new();
    for (key, items) in std::mem::take(&mut *map) {
        for mut item in items {
            let pid = item.child.id();
            if !pid_alive(pid) {
                continue;
            }
            let path = crate::core::project_ref::parse_project_key(&key)
                .map(|(p, _)| p)
                .unwrap_or_else(|| key.clone());
            let kind = item.kind.clone();
            let _ = item.child.kill();
            kill_process_tree(pid);
            stopped_meta.push((path, kind, pid));
        }
    }
    drop(map);
    for (path, kind, pid) in &stopped_meta {
        append_log(
            &state.logs,
            app,
            path,
            kind,
            *pid,
            "[DevKit] 退出前已停止实例".into(),
        );
    }
    Ok(stopped_meta.len() as u32)
}

/// 当前仍存活的托管实例数（按 PID 去重）。
pub fn count_running(state: &AppState) -> u32 {
    let Ok(map) = state.processes.lock() else {
        return 0;
    };
    let mut pids = HashSet::new();
    for items in map.values() {
        for item in items {
            let pid = item.child.id();
            if pid_alive(pid) {
                pids.insert(pid);
            }
        }
    }
    pids.len() as u32
}

pub fn stop_instance(
    app: &AppHandle,
    state: &AppState,
    path: String,
    pid: u32,
) -> Result<(), String> {
    let mut map = state.processes.lock().map_err(|_| "进程状态不可用")?;
    let keys: Vec<String> = map.keys().cloned().collect();
    let mut stopped_kind = String::new();
    for key in keys {
        let is_match = key == path
            || key.starts_with(&format!("{}::", normalize_path(&path)))
            || key == normalize_path(&path);
        if !is_match {
            continue;
        }
        if let Some(items) = map.get_mut(&key) {
            if let Some(pos) = items.iter().position(|r| r.child.id() == pid) {
                stopped_kind = items[pos].kind.clone();
                let mut item = items.remove(pos);
                let _ = item.child.kill();
                kill_process_tree(pid);
            }
            if items.is_empty() {
                map.remove(&key);
            }
        }
    }
    drop(map);
    if !stopped_kind.is_empty() {
        append_log(
            &state.logs,
            app,
            &path,
            &stopped_kind,
            pid,
            format!("[DevKit] 已停止实例 PID {}", pid),
        );
    }
    Ok(())
}

fn pid_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        true
    }
}

fn is_managed_pid(state: &AppState, pid: u32) -> bool {
    let Ok(map) = state.processes.lock() else {
        return false;
    };
    map.values()
        .flatten()
        .any(|r| r.child.id() == pid && pid_alive(pid))
}

/// Host 监控采样：仅允许本应用托管的进程。
pub fn process_metrics(
    state: &AppState,
    pid: u32,
) -> Result<crate::platform::ProcessMetrics, String> {
    if !is_managed_pid(state, pid) {
        return Err(format!("PID {pid} 不是当前托管的运行实例"));
    }
    crate::platform::sample_process(pid)
}

pub fn running_processes(state: &AppState) -> HashMap<String, Vec<ProcessView>> {
    let mut map = state.processes.lock().unwrap();
    let mut out: HashMap<String, Vec<ProcessView>> = HashMap::new();
    for (key, items) in map.iter_mut() {
        for item in items.iter_mut() {
            if !pid_alive(item.child.id()) {
                continue;
            }
            // 优先 OS 真实监听（含子进程）；日志/命令推断仅作无监听时的兜底
            if let Some(real) = port_for_pid(item.child.id()) {
                if concrete_listen_port(&real).is_some() {
                    item.port = Some(real);
                }
            }
        }
        let views: Vec<ProcessView> = items
            .iter()
            .filter(|v| pid_alive(v.child.id()))
            .map(|v| ProcessView {
                pid: v.child.id(),
                action: v.action.clone(),
                port: v.port.clone(),
                kind: v.kind.clone(),
            })
            .collect();
        if views.is_empty() {
            continue;
        }
        // 双写：projectKey + 纯 path（兼容旧前端）
        out.insert(key.clone(), views.clone());
        if let Some((path, _)) = crate::core::project_ref::parse_project_key(key) {
            out.entry(path).or_default().extend(views);
        }
    }
    out
}

pub fn project_logs(state: &AppState, path: String, kind: String) -> Vec<String> {
    let key = make_log_key(&path, &kind);
    state
        .logs
        .lock()
        .unwrap()
        .get(&key)
        .cloned()
        .unwrap_or_default()
}

pub fn clear_logs(state: &AppState, path: String, kind: String) -> Result<(), String> {
    let key = make_log_key(&path, &kind);
    state
        .logs
        .lock()
        .map_err(|_| "日志状态不可用")?
        .insert(key, vec![]);
    Ok(())
}
