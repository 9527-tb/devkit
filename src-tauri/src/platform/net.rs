//! 端口 / 监听探测抽象（Host 监控用）。
//!
//! 工具箱「端口管理」见 `tools::ports`，勿在此混入工具业务。
//!
//! DONE(plat-windows): netstat 解析 — DESIGN §5.2
//! DONE(plat-linux): lsof / 进程组与子进程树 — DESIGN §5.2

use std::collections::HashSet;
use std::process::Command;

/// 列出某 PID 自身正在监听的 TCP 端口。
pub fn list_listening_ports(pid: u32) -> Vec<u16> {
    #[cfg(windows)]
    {
        let output = Command::new("netstat").args(["-ano"]).output().ok();
        return parse_netstat_ports(output.as_ref().map(|o| &o.stdout[..]).unwrap_or(&[]), pid);
    }
    #[cfg(unix)]
    {
        if let Ok(output) = Command::new("lsof")
            .args(["-nP", &format!("-p{pid}"), "-iTCP", "-sTCP:LISTEN"])
            .output()
        {
            if output.status.success() || !output.stdout.is_empty() {
                return parse_lsof_ports(&output.stdout);
            }
        }
        Vec::new()
    }
}

/// 列出托管根进程及其子树（Unix 上含同进程组）的监听端口。
pub fn list_listening_ports_tree(root_pid: u32) -> Vec<u16> {
    #[cfg(unix)]
    {
        if let Ok(output) = Command::new("lsof")
            .args([
                "-nP",
                "-a",
                &format!("-g{root_pid}"),
                "-iTCP",
                "-sTCP:LISTEN",
            ])
            .output()
        {
            if output.status.success() || !output.stdout.is_empty() {
                let ports = parse_lsof_ports(&output.stdout);
                if !ports.is_empty() {
                    return ports;
                }
            }
        }

        let mut ports = Vec::new();
        for pid in descendant_pids_inclusive(root_pid) {
            ports.extend(list_listening_ports(pid));
        }
        ports.sort_unstable();
        ports.dedup();
        return ports;
    }

    #[cfg(windows)]
    {
        let mut ports = Vec::new();
        for pid in descendant_pids_inclusive(root_pid) {
            ports.extend(list_listening_ports(pid));
        }
        ports.sort_unstable();
        ports.dedup();
        ports
    }
}

fn descendant_pids_inclusive(root_pid: u32) -> Vec<u32> {
    let mut out = vec![root_pid];
    let mut seen = HashSet::from([root_pid]);
    let mut queue = vec![root_pid];

    while let Some(parent) = queue.pop() {
        for child in direct_children(parent) {
            if seen.insert(child) {
                out.push(child);
                queue.push(child);
            }
        }
    }
    out
}

#[cfg(unix)]
fn direct_children(parent: u32) -> Vec<u32> {
    let Ok(output) = Command::new("pgrep")
        .args(["-P", &parent.to_string()])
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

#[cfg(windows)]
fn direct_children(parent: u32) -> Vec<u32> {
    let script = format!(
        "Get-CimInstance Win32_Process -Filter \"ParentProcessId={parent}\" | Select-Object -ExpandProperty ProcessId"
    );
    let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

#[cfg(windows)]
fn parse_netstat_ports(stdout: &[u8], pid: u32) -> Vec<u16> {
    let text = String::from_utf8_lossy(stdout);
    let pid_s = pid.to_string();
    let mut ports = Vec::new();
    for line in text.lines() {
        if !line.contains("LISTENING") {
            continue;
        }
        let cols: Vec<_> = line.split_whitespace().collect();
        if cols.len() < 5 {
            continue;
        }
        if cols[cols.len() - 1] != pid_s {
            continue;
        }
        if let Some(port) = cols[1].rsplit(':').next().and_then(|p| p.parse().ok()) {
            ports.push(port);
        }
    }
    ports.sort_unstable();
    ports.dedup();
    ports
}

#[cfg(unix)]
fn parse_lsof_ports(stdout: &[u8]) -> Vec<u16> {
    let text = String::from_utf8_lossy(stdout);
    let mut ports = Vec::new();
    for line in text.lines().skip(1) {
        if let Some(idx) = line.rfind(':') {
            let rest = &line[idx + 1..];
            let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(p) = num.parse::<u16>() {
                ports.push(p);
            }
        }
    }
    ports.sort_unstable();
    ports.dedup();
    ports
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn parse_lsof_extracts_ports() {
        let sample = b"COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME\njava    1234 me   50u  IPv6 0x0      0t0  TCP *:18080 (LISTEN)\n";
        assert_eq!(parse_lsof_ports(sample), vec![18080]);
    }
}
