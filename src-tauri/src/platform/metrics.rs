//! 进程 Host 采样：CPU / 内存 / 磁盘 IO / 命令行 / 环境等。
//!
//! 对应 DESIGN.md §8.6 Monitor Host 通用采样。

use crate::platform::net::list_listening_ports_tree;
use serde::Serialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{
    CpuRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMetrics {
    pub pid: u32,
    /// 相对单核归一化后的占用（约 0..=100）
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub virtual_memory_bytes: u64,
    pub disk_read_bps: u64,
    pub disk_write_bps: u64,
    pub name: String,
    pub cmd: String,
    pub exe: String,
    pub cwd: String,
    pub parent_pid: Option<u32>,
    pub status: String,
    pub start_time: u64,
    pub run_time: u64,
    pub user: String,
    pub ports: Vec<u16>,
    pub environ: Vec<(String, String)>,
}

struct SampleState {
    system: System,
    last_at: HashMap<u32, Instant>,
}

impl SampleState {
    fn new() -> Self {
        Self {
            system: System::new(),
            last_at: HashMap::new(),
        }
    }
}

static STATE: Mutex<Option<SampleState>> = Mutex::new(None);

fn refresh_kind() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing()
        .with_cpu()
        .with_memory()
        .with_disk_usage()
        .with_cmd(UpdateKind::Always)
        .with_exe(UpdateKind::Always)
        .with_cwd(UpdateKind::Always)
        .with_user(UpdateKind::Always)
        .with_environ(UpdateKind::Always)
}

fn os_to_string(s: impl AsRef<OsStr>) -> String {
    s.as_ref().to_string_lossy().into_owned()
}

/// 采样指定 PID。首次会双采样以拿到有效 CPU。
pub fn sample_process(pid: u32) -> Result<ProcessMetrics, String> {
    let mut guard = STATE.lock().map_err(|_| "监控状态不可用")?;
    let state = guard.get_or_insert_with(SampleState::new);
    let sys_pid = Pid::from_u32(pid);
    let kind = refresh_kind();

    if state.system.cpus().is_empty() {
        state
            .system
            .refresh_cpu_list(CpuRefreshKind::nothing());
    }

    // 复用 System：相邻两次采样（前端约 2s）之间的差分即可得到 CPU / 磁盘速率
    let first_sample = !state.last_at.contains_key(&pid);
    state
        .system
        .refresh_processes_specifics(ProcessesToUpdate::Some(&[sys_pid]), true, kind);

    // 首次采样补一次短间隔刷新，否则 sysinfo CPU 几乎恒为 0
    if first_sample {
        thread::sleep(Duration::from_millis(80));
        state
            .system
            .refresh_processes_specifics(ProcessesToUpdate::Some(&[sys_pid]), true, kind);
    }

    let cpu_count = state.system.cpus().len().max(1) as f32;

    let proc = state
        .system
        .process(sys_pid)
        .ok_or_else(|| format!("进程 {pid} 不存在或已退出"))?;

    let now = Instant::now();
    let prev = state.last_at.insert(pid, now);
    let elapsed = prev
        .map(|t| now.saturating_duration_since(t).as_secs_f64())
        .unwrap_or(0.0);

    let disk = proc.disk_usage();
    // 首次无基线，磁盘速率置 0，避免用极小 elapsed 放大累计字节
    let (disk_read_bps, disk_write_bps) = if first_sample || elapsed < 0.05 {
        (0, 0)
    } else {
        (
            (disk.read_bytes as f64 / elapsed).round() as u64,
            (disk.written_bytes as f64 / elapsed).round() as u64,
        )
    };
    let cpu_percent = (proc.cpu_usage() / cpu_count).clamp(0.0, 100.0);

    let cmd = proc
        .cmd()
        .iter()
        .map(os_to_string)
        .collect::<Vec<_>>()
        .join(" ");

    let exe = proc
        .exe()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let cwd = proc
        .cwd()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    let user = proc
        .user_id()
        .map(|u| u.to_string())
        .unwrap_or_else(|| "—".into());

    let mut environ: Vec<(String, String)> = proc
        .environ()
        .iter()
        .filter_map(|entry| {
            let s = os_to_string(entry);
            let (k, v) = s.split_once('=')?;
            Some((k.to_string(), v.to_string()))
        })
        .collect();

    const PREFER: &[&str] = &[
        "JAVA_HOME",
        "JAVA_TOOL_OPTIONS",
        "MAVEN_OPTS",
        "MAVEN_HOME",
        "NODE_ENV",
        "PATH",
        "PWD",
        "HOME",
        "USER",
        "LANG",
        "NPM_CONFIG_USER_AGENT",
    ];
    environ.sort_by(|a, b| {
        let ai = PREFER.iter().position(|k| *k == a.0).unwrap_or(999);
        let bi = PREFER.iter().position(|k| *k == b.0).unwrap_or(999);
        ai.cmp(&bi).then_with(|| a.0.cmp(&b.0))
    });
    if environ.len() > 48 {
        environ.truncate(48);
    }

    // 含子进程树：mvn/npm 父进程通常无 LISTEN，端口在 java/node 子进程上
    let ports = list_listening_ports_tree(pid);

    Ok(ProcessMetrics {
        pid,
        cpu_percent,
        memory_bytes: proc.memory(),
        virtual_memory_bytes: proc.virtual_memory(),
        disk_read_bps,
        disk_write_bps,
        name: os_to_string(proc.name()),
        cmd: if cmd.is_empty() {
            exe.clone()
        } else {
            cmd
        },
        exe,
        cwd,
        parent_pid: proc.parent().map(|p| p.as_u32()),
        status: format!("{:?}", proc.status()),
        start_time: proc.start_time(),
        run_time: proc.run_time(),
        user,
        ports,
        environ,
    })
}
