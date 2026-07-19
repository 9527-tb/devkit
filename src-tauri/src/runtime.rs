use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEntry {
    pub id: String,
    pub version: String,
    pub path: String,
    pub label: String,
    #[serde(default)]
    pub manual: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GeneralSettings {
    #[serde(default = "default_true")]
    pub log_wrap: bool,
    /// 任务完成时系统通知（默认开启）
    #[serde(default = "default_true")]
    pub notify_on_task_done: bool,
    /// 扫描后自动环境体检（默认开启）
    #[serde(default = "default_true")]
    pub health_check_on_scan: bool,
    /// 优先使用工作区 `.devkit/workspace.json` 中的可分享配置
    #[serde(default = "default_true")]
    pub prefer_workspace_config: bool,
    /// 默认编辑器：cursor / code / idea 等（空=探测本机默认）
    #[serde(default)]
    pub editor_command: String,
    /// 默认控制台：terminal / iterm / warp / wt / cmd 等（空=探测本机默认）
    #[serde(default)]
    pub terminal_app: String,
    /// teal | ocean | forest | slate | amber | rose
    #[serde(default = "default_theme")]
    pub theme: String,
    /// zh-CN | zh-TW | en | ja
    #[serde(default = "default_locale")]
    pub locale: String,
    /// 工作台功能按钮默认展示数量（1..=10）
    #[serde(default = "default_action_button_count")]
    pub action_button_count: u32,
    /// 开机自动启动（默认关闭）
    #[serde(default)]
    pub launch_at_login: bool,
    /// 点击关闭：true=最小化到托盘（默认），false=退出应用
    #[serde(default = "default_true")]
    pub close_to_tray: bool,
    /// 启动后静默检查更新（默认开启）
    #[serde(default = "default_true")]
    pub auto_check_update: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            log_wrap: true,
            notify_on_task_done: true,
            health_check_on_scan: true,
            prefer_workspace_config: true,
            editor_command: String::new(),
            terminal_app: String::new(),
            theme: default_theme(),
            locale: default_locale(),
            action_button_count: default_action_button_count(),
            launch_at_login: false,
            close_to_tray: true,
            auto_check_update: true,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_theme() -> String {
    "teal".into()
}

fn default_locale() -> String {
    "zh-CN".into()
}

fn default_action_button_count() -> u32 {
    5
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JavaSettings {
    #[serde(default)]
    pub jdks: Vec<RuntimeEntry>,
    #[serde(default)]
    pub maven_home: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NodeSettings {
    #[serde(default)]
    pub nodes: Vec<RuntimeEntry>,
    /// 执行 Node 项目动作时使用的包管理器：npm | pnpm | yarn（默认 npm）
    #[serde(default = "default_node_package_manager")]
    pub package_manager: String,
}

impl Default for NodeSettings {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            package_manager: default_node_package_manager(),
        }
    }
}

fn default_node_package_manager() -> String {
    "npm".into()
}

/// 规范化包管理器名称；非法值回退 npm。
pub fn normalize_node_package_manager(raw: &str) -> &'static str {
    match raw.trim().to_ascii_lowercase().as_str() {
        "pnpm" => "pnpm",
        "yarn" => "yarn",
        _ => "npm",
    }
}

/// 探测系统已安装的 Node 包管理器（npm / pnpm / yarn），供设置页下拉选择。
/// 会在补全 PATH（nvm / Homebrew / volta 等）中查找。
pub fn detect_node_package_managers() -> Vec<String> {
    const CANDIDATES: &[&str] = &["npm", "pnpm", "yarn"];
    let mut found = Vec::new();
    for name in CANDIDATES {
        if crate::platform::find_executable(name).is_some() {
            found.push((*name).to_string());
        }
    }
    found
}

/// 单 Kind 的全局侧栏过滤：勾选的 actions 为过滤条件（具备其一即保留）；空 = 不过滤。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct KindFilterRule {
    #[serde(default)]
    pub actions: Vec<String>,
    /// Maven：仅保留 `<packaging>jar</packaging>`（缺省 packaging 视为 jar）
    /// 旧配置无此字段时默认开启，与前端 normalize 一致。
    #[serde(default = "default_true")]
    pub only_jar: bool,
}

impl Default for KindFilterRule {
    fn default() -> Self {
        Self {
            actions: Vec::new(),
            only_jar: false,
        }
    }
}

fn default_maven_filter() -> KindFilterRule {
    KindFilterRule {
        actions: vec![
            "spring-boot:run".into(),
            "package".into(),
            "install".into(),
            "clean install".into(),
            "clean".into(),
        ],
        only_jar: true,
    }
}

fn default_node_filter() -> KindFilterRule {
    KindFilterRule {
        actions: vec![
            "script:start".into(),
            "script:dev".into(),
            "script:build".into(),
            "script:lint".into(),
            "install".into(),
        ],
        only_jar: false,
    }
}

fn default_cargo_filter() -> KindFilterRule {
    KindFilterRule {
        actions: vec![
            "run".into(),
            "build".into(),
            "test".into(),
            "check".into(),
            "clean".into(),
        ],
        only_jar: false,
    }
}

fn default_gradle_filter() -> KindFilterRule {
    KindFilterRule {
        actions: vec![
            "bootRun".into(),
            "build".into(),
            "test".into(),
            "clean".into(),
        ],
        only_jar: false,
    }
}

/// 全局项目过滤（与工作区 / 当前选中无关）。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectFilterSettings {
    #[serde(default = "default_maven_filter")]
    pub maven: KindFilterRule,
    #[serde(default = "default_node_filter")]
    pub node: KindFilterRule,
    #[serde(default = "default_cargo_filter")]
    pub cargo: KindFilterRule,
    #[serde(default = "default_gradle_filter")]
    pub gradle: KindFilterRule,
}

impl Default for ProjectFilterSettings {
    fn default() -> Self {
        Self {
            maven: default_maven_filter(),
            node: default_node_filter(),
            cargo: default_cargo_filter(),
            gradle: default_gradle_filter(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSettings {
    #[serde(default)]
    pub general: GeneralSettings,
    #[serde(default)]
    pub java: JavaSettings,
    #[serde(default)]
    pub node: NodeSettings,
    /// 全局侧栏项目过滤
    #[serde(default)]
    pub project_filter: ProjectFilterSettings,
    // legacy flat fields (migrate on load)
    #[serde(default, skip_serializing)]
    pub jdks: Vec<RuntimeEntry>,
    #[serde(default, skip_serializing)]
    pub nodes: Vec<RuntimeEntry>,
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DetectedRuntimes {
    pub jdks: Vec<RuntimeEntry>,
    pub nodes: Vec<RuntimeEntry>,
}

fn settings_path(_app: &AppHandle) -> Result<PathBuf, String> {
    // 统一经 platform 取配置路径（Windows = %APPDATA%\devkit）
    crate::platform::settings_file()
}

pub fn settings_file_exists(app: &AppHandle) -> Result<bool, String> {
    Ok(settings_path(app)?.is_file())
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InitProgress {
    /// prepare | scanJdk | scanNode | scanMaven | writeConfig | done
    pub step: String,
    pub percent: u8,
    pub found: Option<u32>,
    pub detail: Option<String>,
    pub done: bool,
}

fn emit_init_progress(
    app: &AppHandle,
    step: &str,
    percent: u8,
    found: Option<u32>,
    detail: Option<String>,
    done: bool,
) {
    let _ = app.emit(
        "init-progress",
        InitProgress {
            step: step.into(),
            percent,
            found,
            detail,
            done,
        },
    );
}

fn migrate_settings(mut s: RuntimeSettings) -> RuntimeSettings {
    if s.java.jdks.is_empty() && !s.jdks.is_empty() {
        s.java.jdks = s.jdks.clone();
    }
    if s.node.nodes.is_empty() && !s.nodes.is_empty() {
        s.node.nodes = s.nodes.clone();
    }
    s.jdks.clear();
    s.nodes.clear();
    s.general.action_button_count = s.general.action_button_count.clamp(1, 10);
    s
}

pub fn load_settings(app: &AppHandle) -> Result<RuntimeSettings, String> {
    let path = settings_path(app)?;
    if !path.is_file() {
        // First launch is handled by initialize_runtime_settings (UI progress modal).
        return Ok(RuntimeSettings::default());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let parsed: RuntimeSettings =
        serde_json::from_str(&text).map_err(|e| format!("读取配置失败: {e}"))?;
    let mut settings = migrate_settings(parsed);
    // Existing file but Maven Home never set: auto-fill once
    if settings.java.maven_home.trim().is_empty() {
        if let Some(home) = detect_maven_home() {
            settings.java.maven_home = home;
            let _ = save_settings(app, &settings);
        }
    }
    Ok(settings)
}

/// First-launch setup: scan runtimes, write ~/.devkit/settings.json, emit progress events.
pub fn initialize_runtime_settings(app: &AppHandle) -> Result<RuntimeSettings, String> {
    let path = settings_path(app)?;
    if path.is_file() {
        emit_init_progress(app, "done", 100, None, None, true);
        return load_settings(app);
    }

    emit_init_progress(app, "prepare", 8, None, Some(path.to_string_lossy().into()), false);
    thread::sleep(Duration::from_millis(280));

    emit_init_progress(app, "scanJdk", 22, None, None, false);
    let jdks = detect_jdks();
    emit_init_progress(
        app,
        "scanJdk",
        40,
        Some(jdks.len() as u32),
        None,
        false,
    );
    thread::sleep(Duration::from_millis(220));

    emit_init_progress(app, "scanNode", 48, None, None, false);
    let nodes = detect_nodes();
    emit_init_progress(
        app,
        "scanNode",
        65,
        Some(nodes.len() as u32),
        None,
        false,
    );
    thread::sleep(Duration::from_millis(220));

    emit_init_progress(app, "scanMaven", 72, None, None, false);
    let maven_home = detect_maven_home().unwrap_or_default();
    emit_init_progress(
        app,
        "scanMaven",
        84,
        if maven_home.is_empty() { Some(0) } else { Some(1) },
        if maven_home.is_empty() {
            None
        } else {
            Some(maven_home.clone())
        },
        false,
    );
    thread::sleep(Duration::from_millis(220));

    emit_init_progress(app, "writeConfig", 90, None, Some(path.to_string_lossy().into()), false);
    let settings = RuntimeSettings {
        general: GeneralSettings::default(),
        java: JavaSettings {
            jdks,
            maven_home,
        },
        node: NodeSettings {
            nodes,
            ..Default::default()
        },
        ..Default::default()
    };
    save_settings(app, &settings)?;
    thread::sleep(Duration::from_millis(200));

    emit_init_progress(app, "done", 100, None, None, true);
    Ok(settings)
}

pub fn save_settings(app: &AppHandle, settings: &RuntimeSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let mut clean = settings.clone();
    clean.jdks.clear();
    clean.nodes.clear();
    let text = serde_json::to_string_pretty(&clean).map_err(|e| format!("序列化配置失败: {e}"))?;
    fs::write(&path, text).map_err(|e| e.to_string())?;
    sync_launch_at_login(app, settings.general.launch_at_login)?;
    Ok(())
}

/// 将「开机自启」配置同步到系统（LaunchAgent / 注册表等）。
pub fn sync_launch_at_login(app: &AppHandle, enabled: bool) -> Result<(), String> {
    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;
        let launcher = app.autolaunch();
        let result = if enabled {
            launcher.enable()
        } else {
            launcher.disable()
        };
        if let Err(e) = result {
            // 预览/权限不足时不阻断配置保存
            eprintln!("[DevKit] sync launch_at_login failed: {e}");
        }
    }
    let _ = (app, enabled);
    Ok(())
}

/// 关闭按钮是否应最小化到托盘。
pub fn close_to_tray_enabled(app: &AppHandle) -> bool {
    load_settings(app)
        .map(|s| s.general.close_to_tray)
        .unwrap_or(true)
}

fn looks_like_maven_home(home: &Path) -> bool {
    home.join("bin/mvn").is_file() || home.join("bin/mvn.cmd").is_file()
}

pub fn detect_maven_home() -> Option<String> {
    if let Ok(home) = std::env::var("MAVEN_HOME").or_else(|_| std::env::var("M2_HOME")) {
        let p = PathBuf::from(&home);
        if looks_like_maven_home(&p) {
            return Some(p.to_string_lossy().into());
        }
    }

    // Prefer `mvn -v` "Maven home:" line (works with Homebrew Cellar layouts)
    if let Ok(output) = Command::new("mvn").args(["-v"]).output() {
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        for line in text.lines() {
            let line = line.trim();
            if let Some(rest) = line
                .strip_prefix("Maven home:")
                .or_else(|| line.strip_prefix("Maven home："))
            {
                let home = PathBuf::from(rest.trim());
                if looks_like_maven_home(&home) {
                    return Some(home.to_string_lossy().into());
                }
            }
        }
    }

    if let Ok(output) = Command::new("which").arg("mvn").output() {
        if output.status.success() {
            let mvn = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let mvn = PathBuf::from(mvn);
            let resolved = mvn.canonicalize().unwrap_or(mvn);
            if let Some(bin) = resolved.parent() {
                if let Some(home) = bin.parent() {
                    if looks_like_maven_home(home) {
                        return Some(home.to_string_lossy().into());
                    }
                    let libexec = home.join("libexec");
                    if looks_like_maven_home(&libexec) {
                        return Some(libexec.to_string_lossy().into());
                    }
                }
            }
        }
    }

    for cand in [
        "/opt/homebrew/opt/maven/libexec",
        "/usr/local/opt/maven/libexec",
        "/usr/share/maven",
        "/opt/maven",
    ] {
        let p = PathBuf::from(cand);
        if looks_like_maven_home(&p) {
            return Some(cand.into());
        }
    }
    None
}

pub fn maven_bin(settings: &RuntimeSettings) -> String {
    let home = settings.java.maven_home.trim();
    if !home.is_empty() {
        let mvn = PathBuf::from(home).join("bin/mvn");
        if mvn.is_file() {
            return mvn.to_string_lossy().into();
        }
    }
    "mvn".into()
}

fn entry_id(kind: &str, version: &str, path: &str) -> String {
    format!("{kind}:{version}:{}", path.replace('/', "_"))
}

fn normalize_java_version(raw: &str) -> Option<String> {
    let s = raw.trim().trim_matches('"').trim_matches('\'');
    if s.is_empty() {
        return None;
    }
    if let Some(rest) = s.strip_prefix("1.") {
        let major = rest.split(|c: char| !c.is_ascii_digit()).next().unwrap_or("");
        if major == "8" {
            return Some("1.8".into());
        }
    }
    let major = s.split(|c: char| !c.is_ascii_digit()).next().unwrap_or("");
    if major.is_empty() {
        None
    } else {
        Some(major.to_string())
    }
}

fn normalize_node_version(raw: &str) -> Option<String> {
    let s = raw
        .trim()
        .trim_start_matches('v')
        .trim_matches('"')
        .trim_matches('\'');
    if s.is_empty() {
        return None;
    }
    let cleaned = s
        .trim_start_matches('>')
        .trim_start_matches('=')
        .trim_start_matches('<')
        .trim_start_matches('~')
        .trim_start_matches('^')
        .trim();
    let token = cleaned
        .split(|c: char| c == '|' || c == ' ' || c == ',' || c == '-')
        .find(|t| t.chars().any(|c| c.is_ascii_digit()))
        .unwrap_or(cleaned);
    let major = token.split(|c: char| !c.is_ascii_digit()).next().unwrap_or("");
    if major.is_empty() {
        None
    } else {
        Some(major.to_string())
    }
}

/// 项目声明的 Node 版本需求。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeVersionReq {
    /// 固定主版本（.nvmrc / engines: "20" / "^20"）
    Major(u32),
    /// 最低主版本（engines: ">=20.0.0"）— 选用已安装中满足条件的最新版
    AtLeast(u32),
}

fn parse_node_major_u32(raw: &str) -> Option<u32> {
    normalize_node_version(raw)?.parse().ok()
}

/// 解析 .nvmrc / engines.node 等原始约束。
pub fn parse_node_version_req(raw: &str) -> Option<NodeVersionReq> {
    let s = raw.trim().trim_matches('"').trim_matches('\'');
    if s.is_empty() {
        return None;
    }
    let lower = s.to_ascii_lowercase();
    if lower == "*" || lower == "latest" || lower == "current" {
        return Some(NodeVersionReq::AtLeast(0));
    }

    let body = s.trim_start_matches('v').trim();
    if let Some(rest) = body.strip_prefix(">=") {
        return Some(NodeVersionReq::AtLeast(parse_node_major_u32(rest)?));
    }
    if let Some(rest) = body.strip_prefix('>') {
        let major = parse_node_major_u32(rest)?;
        return Some(NodeVersionReq::AtLeast(major.saturating_add(1)));
    }
    // ^20 / ~20 / 20.x → 固定在该主版本线
    if body.starts_with('^') || body.starts_with('~') {
        return Some(NodeVersionReq::Major(parse_node_major_u32(body)?));
    }
    Some(NodeVersionReq::Major(parse_node_major_u32(body)?))
}

fn node_entry_major(entry: &RuntimeEntry) -> Option<u32> {
    entry.version.parse().ok().or_else(|| parse_node_major_u32(&entry.version))
}

fn semver_tuple_from_path(path: &str) -> Option<(u32, u32, u32)> {
    let re = Regex::new(r"v?(\d+)\.(\d+)\.(\d+)").ok()?;
    let caps = re.captures(path)?;
    Some((
        caps.get(1)?.as_str().parse().ok()?,
        caps.get(2)?.as_str().parse().ok()?,
        caps.get(3)?.as_str().parse().ok()?,
    ))
}

fn node_entry_semver_key(entry: &RuntimeEntry) -> (u32, u32, u32) {
    if let Some(v) = semver_tuple_from_path(&entry.path) {
        return v;
    }
    let major = node_entry_major(entry).unwrap_or(0);
    (major, 0, 0)
}

fn node_entry_bin_ok(entry: &RuntimeEntry) -> bool {
    let home = Path::new(&entry.path);
    home.join("bin/node").is_file() || home.join("node").is_file()
}

/// 按需求从已配置 Node 中挑选最佳运行时（同条件取最新）。
fn pick_node_entry<'a>(
    entries: &'a [RuntimeEntry],
    req: &NodeVersionReq,
) -> Option<&'a RuntimeEntry> {
    let mut matched: Vec<&RuntimeEntry> = entries
        .iter()
        .filter(|e| {
            if !node_entry_bin_ok(e) {
                return false;
            }
            let Some(major) = node_entry_major(e) else {
                return false;
            };
            match req {
                NodeVersionReq::Major(m) => major == *m,
                NodeVersionReq::AtLeast(m) => major >= *m,
            }
        })
        .collect();
    matched.sort_by(|a, b| node_entry_semver_key(b).cmp(&node_entry_semver_key(a)));
    matched.into_iter().next()
}

fn read_text_file(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

/// Real JDK home has bin/java plus release/lib (reject macOS /usr stub).
fn is_valid_jdk_home(home: &Path) -> bool {
    let java = if cfg!(windows) {
        home.join("bin").join("java.exe")
    } else {
        home.join("bin").join("java")
    };
    if !java.is_file() {
        return false;
    }
    let path = home.to_string_lossy();
    // /usr/bin/java is a macOS stub; parent chain wrongly yields /usr
    if path == "/usr" || path == "/usr/local" || path == "/" {
        return false;
    }
    if path.starts_with("/System/") {
        return false;
    }
    home.join("release").is_file()
        || home.join("lib").is_dir()
        || home.join("jre").is_dir()
}

fn probe_java_version(home: &Path) -> Option<String> {
    if !is_valid_jdk_home(home) {
        return None;
    }
    let java = if cfg!(windows) {
        home.join("bin").join("java.exe")
    } else {
        home.join("bin").join("java")
    };
    let output = Command::new(java).arg("-version").output().ok()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let re = Regex::new(r#"version\s+\"([^\"]+)\""#).ok()?;
    let caps = re.captures(&text)?;
    normalize_java_version(caps.get(1)?.as_str())
}

fn probe_node_version(node_bin_or_home: &Path) -> Option<(String, PathBuf)> {
    let bin = if node_bin_or_home
        .file_name()
        .and_then(|s| s.to_str())
        .is_some_and(|n| n == "node" || n == "node.exe")
    {
        node_bin_or_home.to_path_buf()
    } else if node_bin_or_home.join("bin/node").is_file() {
        node_bin_or_home.join("bin/node")
    } else if node_bin_or_home.join("node").is_file() {
        node_bin_or_home.join("node")
    } else {
        return None;
    };
    let output = Command::new(&bin).arg("-v").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let ver = String::from_utf8_lossy(&output.stdout);
    let version = normalize_node_version(ver.trim())?;
    let home = bin
        .parent()
        .and_then(|p| {
            if p.file_name().and_then(|s| s.to_str()) == Some("bin") {
                p.parent().map(|x| x.to_path_buf())
            } else {
                Some(p.to_path_buf())
            }
        })
        .unwrap_or_else(|| bin.clone());
    Some((version, home))
}

fn push_jdk(out: &mut Vec<RuntimeEntry>, seen: &mut HashSet<String>, home: PathBuf, source: &str) {
    let home = if home.join("Contents/Home/bin/java").is_file() {
        home.join("Contents/Home")
    } else {
        home
    };
    let canon = home.canonicalize().unwrap_or(home);
    if !is_valid_jdk_home(&canon) {
        return;
    }
    let path = canon.to_string_lossy().to_string();
    if !seen.insert(path.clone()) {
        return;
    }
    let version = probe_java_version(&canon).unwrap_or_else(|| "?".into());
    let label = if source.is_empty() {
        format!("JDK {version}")
    } else {
        format!("JDK {version} ({source})")
    };
    out.push(RuntimeEntry {
        id: entry_id("jdk", &version, &path),
        version,
        path,
        label,
        manual: false,
    });
}

fn push_node(out: &mut Vec<RuntimeEntry>, seen: &mut HashSet<String>, candidate: PathBuf) {
    let Some((version, home)) = probe_node_version(&candidate) else {
        return;
    };
    let path = home.canonicalize().unwrap_or(home).to_string_lossy().to_string();
    if !seen.insert(path.clone()) {
        return;
    }
    out.push(RuntimeEntry {
        id: entry_id("node", &version, &path),
        version: version.clone(),
        path,
        label: format!("Node {version}"),
        manual: false,
    });
}

fn scan_jvm_root(out: &mut Vec<RuntimeEntry>, seen: &mut HashSet<String>, root: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if !p.is_dir() {
            continue;
        }
        if p.join("Contents/Home/bin/java").is_file() {
            push_jdk(out, seen, p.join("Contents/Home"), "");
        } else if p.join("bin/java").is_file() {
            push_jdk(out, seen, p, "");
        }
    }
}

pub fn detect_jdks() -> Vec<RuntimeEntry> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    #[cfg(target_os = "macos")]
    {
        // Preferred: macOS JVM registry (lists all installs on stderr)
        if let Ok(output) = Command::new("/usr/libexec/java_home").args(["-V"]).output() {
            let text = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stderr),
                String::from_utf8_lossy(&output.stdout)
            );
            let path_re = Regex::new(r"(/[^\s]+/Contents/Home)\s*$").ok();
            for line in text.lines() {
                if let Some(re) = path_re.as_ref() {
                    if let Some(caps) = re.captures(line) {
                        push_jdk(
                            &mut out,
                            &mut seen,
                            PathBuf::from(caps.get(1).unwrap().as_str()),
                            "",
                        );
                        continue;
                    }
                }
                if let Some(idx) = line.rfind(" /") {
                    let path = line[idx + 1..].trim();
                    if path.starts_with('/') && path.contains("Java") {
                        push_jdk(&mut out, &mut seen, PathBuf::from(path), "");
                    }
                }
            }
        }
        scan_jvm_root(
            &mut out,
            &mut seen,
            Path::new("/Library/Java/JavaVirtualMachines"),
        );
        if let Ok(home) = std::env::var("HOME") {
            scan_jvm_root(
                &mut out,
                &mut seen,
                &PathBuf::from(home).join("Library/Java/JavaVirtualMachines"),
            );
        }
    }

    if let Ok(home) = std::env::var("JAVA_HOME") {
        push_jdk(&mut out, &mut seen, PathBuf::from(home), "JAVA_HOME");
    }

    if let Ok(home) = std::env::var("HOME") {
        let home = PathBuf::from(home);
        let sdkman = home.join(".sdkman/candidates/java");
        if let Ok(entries) = fs::read_dir(sdkman) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() && p.file_name().and_then(|n| n.to_str()) != Some("current") {
                    push_jdk(&mut out, &mut seen, p, "sdkman");
                }
            }
        }
        for rel in [".jenv/versions", ".asdf/installs/java", ".jabba/jdk"] {
            scan_jvm_root(&mut out, &mut seen, &home.join(rel));
        }
    }

    for root in ["/opt/homebrew/opt", "/usr/local/opt"] {
        if let Ok(entries) = fs::read_dir(root) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_lowercase();
                if !(name.contains("openjdk")
                    || name.contains("jdk")
                    || name.contains("temurin")
                    || name.contains("liberica")
                    || name.contains("zulu")
                    || name.contains("java"))
                {
                    continue;
                }
                let p = e.path();
                if p.join("libexec/openjdk.jdk/Contents/Home/bin/java").is_file() {
                    push_jdk(
                        &mut out,
                        &mut seen,
                        p.join("libexec/openjdk.jdk/Contents/Home"),
                        "homebrew",
                    );
                } else if p.join("Contents/Home/bin/java").is_file() {
                    push_jdk(&mut out, &mut seen, p.join("Contents/Home"), "homebrew");
                } else if p.join("bin/java").is_file() {
                    push_jdk(&mut out, &mut seen, p, "homebrew");
                }
            }
        }
    }

    // Skip macOS stub /usr/bin/java (would incorrectly become JAVA_HOME=/usr)
    if let Ok(output) = Command::new("which").arg("java").output() {
        if output.status.success() {
            let java = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if java != "/usr/bin/java" && java != "/bin/java" {
                let java_path = PathBuf::from(&java);
                if let Some(bin) = java_path.parent() {
                    if let Some(home) = bin.parent() {
                        push_jdk(&mut out, &mut seen, home.to_path_buf(), "PATH");
                    }
                }
            }
        }
    }

    out.sort_by(|a, b| b.version.cmp(&a.version).then(a.path.cmp(&b.path)));
    out
}

pub fn detect_nodes() -> Vec<RuntimeEntry> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    if let Ok(output) = Command::new("which").arg("node").output() {
        if output.status.success() {
            let node = String::from_utf8_lossy(&output.stdout).trim().to_string();
            push_node(&mut out, &mut seen, PathBuf::from(node));
        }
    }

    let home = std::env::var("HOME").unwrap_or_default();
    if !home.is_empty() {
        let home = PathBuf::from(home);
        let roots = [
            home.join(".nvm/versions/node"),
            home.join(".fnm/node-versions"),
            home.join(".local/share/fnm/node-versions"),
            home.join(".asdf/installs/node"),
            home.join(".volta/tools/image/node"),
        ];
        for root in roots {
            if let Ok(entries) = fs::read_dir(&root) {
                for e in entries.flatten() {
                    let p = e.path();
                    if !p.is_dir() {
                        continue;
                    }
                    if p.join("bin/node").is_file() {
                        push_node(&mut out, &mut seen, p);
                    } else if p.join("installation/bin/node").is_file() {
                        push_node(&mut out, &mut seen, p.join("installation"));
                    }
                }
            }
        }
    }

    for root in ["/opt/homebrew/Cellar/node", "/usr/local/Cellar/node"] {
        if let Ok(entries) = fs::read_dir(root) {
            for e in entries.flatten() {
                let p = e.path();
                if p.join("bin/node").is_file() {
                    push_node(&mut out, &mut seen, p);
                }
            }
        }
    }

    out.sort_by(|a, b| b.version.cmp(&a.version).then(a.path.cmp(&b.path)));
    out
}

pub fn detect_all() -> DetectedRuntimes {
    DetectedRuntimes {
        jdks: detect_jdks(),
        nodes: detect_nodes(),
    }
}

fn version_from_java_version_file(dir: &Path) -> Option<String> {
    let text = read_text_file(&dir.join(".java-version"))?;
    normalize_java_version(text.lines().next().unwrap_or(""))
}

fn version_from_sdkmanrc(dir: &Path) -> Option<String> {
    let text = read_text_file(&dir.join(".sdkmanrc"))?;
    for line in text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("java=") {
            let ver = v.split('-').next().unwrap_or(v);
            return normalize_java_version(ver);
        }
    }
    None
}

fn version_from_tool_versions(dir: &Path, tool: &str) -> Option<String> {
    let text = read_text_file(&dir.join(".tool-versions"))?;
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        if parts.next() == Some(tool) {
            let v = parts.next().unwrap_or("");
            return if tool == "java" {
                normalize_java_version(v)
            } else {
                normalize_node_version(v)
            };
        }
    }
    None
}

fn version_from_eclipse_prefs(dir: &Path) -> Option<String> {
    let text = read_text_file(&dir.join(".settings/org.eclipse.jdt.core.prefs"))?;
    for key in [
        "org.eclipse.jdt.core.compiler.compliance=",
        "org.eclipse.jdt.core.compiler.source=",
    ] {
        for line in text.lines() {
            if let Some(v) = line.trim().strip_prefix(key) {
                return normalize_java_version(v);
            }
        }
    }
    None
}

fn pom_property_map(pom: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(re) = Regex::new(r"<([a-zA-Z0-9_.-]+)\s*>\s*([^<$][^<]*)\s*</") else {
        return map;
    };
    for caps in re.captures_iter(pom) {
        let key = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        let val = caps
            .get(2)
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_default();
        if !key.is_empty() && !val.is_empty() {
            map.entry(key).or_insert(val);
        }
    }
    map
}

fn resolve_pom_value(raw: &str, props: &HashMap<String, String>) -> Option<String> {
    let raw = raw.trim();
    if let Some(inner) = raw.strip_prefix("${").and_then(|s| s.strip_suffix('}')) {
        let v = props.get(inner)?;
        return normalize_java_version(v);
    }
    normalize_java_version(raw)
}

fn version_from_pom_text(pom: &str, props: &HashMap<String, String>) -> Option<String> {
    let patterns = [
        r"<maven\.compiler\.release\s*>\s*([^<]+?)\s*</maven\.compiler\.release>",
        r"<maven\.compiler\.source\s*>\s*([^<]+?)\s*</maven\.compiler\.source>",
        r"<java\.version\s*>\s*([^<]+?)\s*</java\.version>",
    ];
    for pat in patterns {
        if let Ok(re) = Regex::new(pat) {
            if let Some(caps) = re.captures(pom) {
                if let Some(m) = caps.get(1) {
                    if let Some(v) = resolve_pom_value(m.as_str(), props) {
                        return Some(v);
                    }
                }
            }
        }
    }
    for key in ["maven.compiler.release", "maven.compiler.source", "java.version"] {
        if let Some(v) = props.get(key) {
            if let Some(n) = normalize_java_version(v) {
                return Some(n);
            }
        }
    }
    None
}

pub fn detect_project_java_version(project_path: &Path) -> Option<String> {
    let mut dir = project_path.to_path_buf();
    if dir.is_file() {
        dir.pop();
    }
    let mut props = HashMap::new();
    for _ in 0..10 {
        if let Some(v) = version_from_java_version_file(&dir)
            .or_else(|| version_from_sdkmanrc(&dir))
            .or_else(|| version_from_tool_versions(&dir, "java"))
            .or_else(|| version_from_eclipse_prefs(&dir))
        {
            return Some(v);
        }
        if let Some(pom) = read_text_file(&dir.join("pom.xml")) {
            let local = pom_property_map(&pom);
            for (k, v) in local {
                props.entry(k).or_insert(v);
            }
            if let Some(v) = version_from_pom_text(&pom, &props) {
                return Some(v);
            }
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

/// 检测项目 Node 版本需求（保留 >= / 固定主版本语义）。
pub fn detect_project_node_requirement(project_path: &Path) -> Option<NodeVersionReq> {
    let mut dir = project_path.to_path_buf();
    if dir.is_file() {
        dir.pop();
    }
    for _ in 0..10 {
        if let Some(req) = read_text_file(&dir.join(".nvmrc"))
            .and_then(|t| parse_node_version_req(t.lines().next().unwrap_or("")))
            .or_else(|| {
                read_text_file(&dir.join(".node-version"))
                    .and_then(|t| parse_node_version_req(t.lines().next().unwrap_or("")))
            })
            .or_else(|| {
                version_from_tool_versions(&dir, "node")
                    .and_then(|v| parse_node_version_req(&v))
            })
        {
            return Some(req);
        }
        if let Some(pkg) = read_text_file(&dir.join("package.json")) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&pkg) {
                if let Some(req) = json
                    .pointer("/engines/node")
                    .and_then(|v| v.as_str())
                    .and_then(parse_node_version_req)
                {
                    return Some(req);
                }
            }
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

#[allow(dead_code)]
pub fn detect_project_node_version(project_path: &Path) -> Option<String> {
    match detect_project_node_requirement(project_path)? {
        NodeVersionReq::Major(m) | NodeVersionReq::AtLeast(m) => Some(m.to_string()),
    }
}

fn find_runtime<'a>(entries: &'a [RuntimeEntry], version: &str) -> Option<&'a RuntimeEntry> {
    entries
        .iter()
        .find(|e| e.version == version)
        .or_else(|| {
            entries.iter().find(|e| {
                e.version.starts_with(&format!("{version}."))
                    || (version == "1.8" && (e.version == "8" || e.version.starts_with("1.8")))
            })
        })
}

pub fn resolve_jdk(
    settings: &RuntimeSettings,
    project_path: &Path,
) -> Result<(Option<String>, Option<RuntimeEntry>), String> {
    let required = detect_project_java_version(project_path);
    let Some(version) = required else {
        return Ok((None, None));
    };
    if let Some(entry) = find_runtime(&settings.java.jdks, &version) {
        if Path::new(&entry.path).is_dir() {
            return Ok((Some(version), Some(entry.clone())));
        }
        return Err(format!(
            "项目需要 Java {version}，已配置路径不可用：{}\n请打开「设置」重新选择 JDK",
            entry.path
        ));
    }
    Err(format!(
        "项目需要 Java {version}，但设置中未配置对应 JDK。\n请点击右上角「设置」添加 Java {version} 的安装路径。"
    ))
}

pub fn resolve_node(
    settings: &RuntimeSettings,
    project_path: &Path,
) -> Result<(Option<String>, Option<RuntimeEntry>), String> {
    let Some(req) = detect_project_node_requirement(project_path) else {
        return Ok((None, None));
    };
    if let Some(entry) = pick_node_entry(&settings.node.nodes, &req) {
        return Ok((Some(entry.version.clone()), Some(entry.clone())));
    }
    let hint = match req {
        NodeVersionReq::Major(m) => format!("Node {m}"),
        NodeVersionReq::AtLeast(m) => format!("Node >={m}"),
    };
    Err(format!(
        "项目需要 {hint}，但设置中未配置可用的 Node。\n请点击右上角「设置」添加满足要求的 Node 安装路径。"
    ))
}

pub fn validate_manual_jdk(path: &str) -> Result<RuntimeEntry, String> {
    let home = PathBuf::from(path);
    if !home.is_dir() {
        return Err("JDK 路径不存在".into());
    }
    let version = probe_java_version(&home).ok_or("无法识别该路径下的 Java 版本")?;
    let path = home
        .canonicalize()
        .unwrap_or(home)
        .to_string_lossy()
        .to_string();
    Ok(RuntimeEntry {
        id: entry_id("jdk", &version, &path),
        version: version.clone(),
        path,
        label: format!("JDK {version}"),
        manual: true,
    })
}

pub fn validate_manual_node(path: &str) -> Result<RuntimeEntry, String> {
    let candidate = PathBuf::from(path);
    let (version, home) =
        probe_node_version(&candidate).ok_or("无法识别该路径下的 Node 版本")?;
    let path = home
        .canonicalize()
        .unwrap_or(home)
        .to_string_lossy()
        .to_string();
    Ok(RuntimeEntry {
        id: entry_id("node", &version, &path),
        version: version.clone(),
        path,
        label: format!("Node {version}"),
        manual: true,
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_jdks_skips_usr_stub() {
        let jdks = detect_jdks();
        for j in &jdks {
            assert_ne!(j.path, "/usr", "should not treat /usr as JDK: {j:?}");
            assert!(!j.label.contains("which java"), "{j:?}");
        }
        assert!(!jdks.is_empty(), "expected at least one real JDK on this machine");
        println!("{:#?}", jdks);
    }

    #[test]
    fn detect_node_package_managers_finds_common_tools() {
        let found = detect_node_package_managers();
        println!("package managers: {found:?}");
        // 开发机通常至少有 npm；若 CI 无 Node 则跳过断言
        if crate::platform::find_executable("node").is_some() {
            assert!(
                found.iter().any(|x| x == "npm"),
                "expected npm when node is on PATH, got {found:?}"
            );
        }
        for name in &found {
            assert!(
                matches!(name.as_str(), "npm" | "pnpm" | "yarn"),
                "unexpected pm: {name}"
            );
        }
    }

    #[test]
    fn parse_node_version_req_ranges() {
        assert_eq!(
            parse_node_version_req(">=20.0.0"),
            Some(NodeVersionReq::AtLeast(20))
        );
        assert_eq!(
            parse_node_version_req("^20.0.0"),
            Some(NodeVersionReq::Major(20))
        );
        assert_eq!(
            parse_node_version_req("v22.23.1"),
            Some(NodeVersionReq::Major(22))
        );
        assert_eq!(
            parse_node_version_req("20"),
            Some(NodeVersionReq::Major(20))
        );
    }

    #[test]
    fn pick_node_entry_prefers_newest_for_at_least() {
        let root = std::env::temp_dir().join(format!(
            "devkit-node-pick-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let mk = |name: &str| {
            let home = root.join(name);
            fs::create_dir_all(home.join("bin")).unwrap();
            fs::write(home.join("bin/node"), b"#!/bin/sh\n").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(home.join("bin/node"), fs::Permissions::from_mode(0o755));
            }
            home
        };
        let p20 = mk("v20.12.1");
        let p22 = mk("v22.23.1");
        let p24 = mk("v24.18.0");
        let entries = vec![
            RuntimeEntry {
                id: "a".into(),
                version: "20".into(),
                path: p20.to_string_lossy().into(),
                label: "Node 20".into(),
                manual: false,
            },
            RuntimeEntry {
                id: "b".into(),
                version: "22".into(),
                path: p22.to_string_lossy().into(),
                label: "Node 22".into(),
                manual: false,
            },
            RuntimeEntry {
                id: "c".into(),
                version: "24".into(),
                path: p24.to_string_lossy().into(),
                label: "Node 24".into(),
                manual: false,
            },
        ];
        let picked = pick_node_entry(&entries, &NodeVersionReq::AtLeast(20)).unwrap();
        assert_eq!(picked.version, "24");
        let pinned = pick_node_entry(&entries, &NodeVersionReq::Major(20)).unwrap();
        assert_eq!(pinned.version, "20");
        let _ = fs::remove_dir_all(&root);
    }
}
