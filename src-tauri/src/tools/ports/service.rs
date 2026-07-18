//! 端口占用查询与结束进程。

use serde::Serialize;
use std::collections::HashMap;
use std::process::Command;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

/// 占用某端口的进程信息（不含完整命令行，避免噪音与隐私暴露）。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortOccupant {
    pub pid: u32,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub address: String,
    pub protocol: String,
    pub state: String,
}

/// 查询端口占用。
/// - `fuzzy = false`：精确匹配端口号
/// - `fuzzy = true`：端口号字符串包含查询片段（如 `80` → 80 / 8080 / 18080）
pub fn lookup_port(query: &str, fuzzy: bool) -> Result<Vec<PortOccupant>, String> {
    let query = query.trim();
    if query.is_empty() || !query.chars().all(|c| c.is_ascii_digit()) {
        return Err("请输入有效的端口数字".into());
    }
    if query.len() > 5 {
        return Err("端口号无效".into());
    }

    let rows = if fuzzy {
        lookup_fuzzy(query)?
    } else {
        let port: u16 = query
            .parse()
            .map_err(|_| "请输入 1–65535 之间的端口号".to_string())?;
        if port == 0 {
            return Err("端口号无效".into());
        }
        lookup_exact(port)?
    };

    let mut rows = rows;
    enrich_occupants(&mut rows);
    rows.sort_by(|a, b| a.port.cmp(&b.port).then(a.pid.cmp(&b.pid)));
    Ok(rows)
}

fn lookup_exact(port: u16) -> Result<Vec<PortOccupant>, String> {
    #[cfg(unix)]
    {
        let output = Command::new("lsof")
            .args(["-nP", &format!("-iTCP:{port}"), "-sTCP:LISTEN"])
            .output()
            .map_err(|e| format!("执行 lsof 失败: {e}"))?;
        let mut rows = parse_lsof_occupants(&output.stdout, |p| p == port);
        if rows.is_empty() {
            if let Ok(all) = Command::new("lsof")
                .args(["-nP", &format!("-iTCP:{port}")])
                .output()
            {
                rows = parse_lsof_occupants(&all.stdout, |p| p == port);
            }
        }
        Ok(rows)
    }
    #[cfg(windows)]
    {
        let output = Command::new("netstat")
            .args(["-ano"])
            .output()
            .map_err(|e| format!("执行 netstat 失败: {e}"))?;
        Ok(parse_netstat_occupants(&output.stdout, |p| p == port))
    }
}

fn lookup_fuzzy(query: &str) -> Result<Vec<PortOccupant>, String> {
    let needle = query.to_string();
    #[cfg(unix)]
    {
        let output = Command::new("lsof")
            .args(["-nP", "-iTCP", "-sTCP:LISTEN"])
            .output()
            .map_err(|e| format!("执行 lsof 失败: {e}"))?;
        Ok(parse_lsof_occupants(&output.stdout, |p| {
            p.to_string().contains(&needle)
        }))
    }
    #[cfg(windows)]
    {
        let output = Command::new("netstat")
            .args(["-ano"])
            .output()
            .map_err(|e| format!("执行 netstat 失败: {e}"))?;
        Ok(parse_netstat_occupants(&output.stdout, |p| {
            p.to_string().contains(&needle)
        }))
    }
}

pub fn kill_process(pid: u32) -> Result<(), String> {
    crate::platform::kill_pid(pid)
}

fn enrich_occupants(rows: &mut [PortOccupant]) {
    if rows.is_empty() {
        return;
    }
    let mut sys = System::new();
    let kind = ProcessRefreshKind::nothing().with_user(UpdateKind::Always);
    let pids: Vec<Pid> = rows.iter().map(|r| Pid::from_u32(r.pid)).collect();
    sys.refresh_processes_specifics(ProcessesToUpdate::Some(&pids), true, kind);

    for row in rows.iter_mut() {
        if let Some(proc) = sys.process(Pid::from_u32(row.pid)) {
            if row.name.is_empty() || row.name == "—" {
                row.name = proc.name().to_string_lossy().into_owned();
            }
            if let Some(uid) = proc.user_id() {
                row.user = uid.to_string();
            }
        }
    }
}

fn extract_port_from_address(address: &str) -> Option<u16> {
    // *:8080 / 127.0.0.1:8080 / [::1]:8080 (LISTEN)
    let base = address.split_whitespace().next().unwrap_or(address);
    let base = base.trim_end_matches(']');
    let port_part = base.rsplit(':').next()?;
    let digits: String = port_part
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

#[cfg(unix)]
fn parse_lsof_occupants<F>(stdout: &[u8], mut port_match: F) -> Vec<PortOccupant>
where
    F: FnMut(u16) -> bool,
{
    let text = String::from_utf8_lossy(stdout);
    // key: pid + port
    let mut by_key: HashMap<(u32, u16), PortOccupant> = HashMap::new();

    for line in text.lines().skip(1) {
        let cols: Vec<_> = line.split_whitespace().collect();
        if cols.len() < 9 {
            continue;
        }
        let name = cols[0].to_string();
        let Ok(pid) = cols[1].parse::<u32>() else {
            continue;
        };
        let user = cols[2].to_string();
        let address = cols[8].to_string();
        let Some(port) = extract_port_from_address(&address) else {
            continue;
        };
        if !port_match(port) {
            continue;
        }
        let state = if line.contains("(LISTEN)") {
            "LISTEN".into()
        } else if line.contains("(ESTABLISHED)") {
            "ESTABLISHED".into()
        } else {
            "TCP".into()
        };
        by_key.entry((pid, port)).or_insert(PortOccupant {
            pid,
            port,
            name,
            user,
            address,
            protocol: "tcp".into(),
            state,
        });
    }
    by_key.into_values().collect()
}

#[cfg(windows)]
fn parse_netstat_occupants<F>(stdout: &[u8], mut port_match: F) -> Vec<PortOccupant>
where
    F: FnMut(u16) -> bool,
{
    let text = String::from_utf8_lossy(stdout);
    let mut by_key: HashMap<(u32, u16), PortOccupant> = HashMap::new();

    for line in text.lines() {
        let cols: Vec<_> = line.split_whitespace().collect();
        if cols.len() < 4 {
            continue;
        }
        let proto = cols[0].to_lowercase();
        if !proto.starts_with("tcp") {
            continue;
        }
        let local = cols[1];
        let Some(port) = extract_port_from_address(local) else {
            continue;
        };
        if !port_match(port) {
            continue;
        }
        let state = if cols.len() >= 5 {
            cols[3].to_string()
        } else {
            "—".into()
        };
        let pid_s = cols[cols.len() - 1];
        let Ok(pid) = pid_s.parse::<u32>() else {
            continue;
        };
        if pid == 0 {
            continue;
        }
        by_key.entry((pid, port)).or_insert(PortOccupant {
            pid,
            port,
            name: String::new(),
            user: String::new(),
            address: local.to_string(),
            protocol: proto,
            state,
        });
    }
    by_key.into_values().collect()
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn parse_lsof_exact() {
        let sample = b"COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME\njava    1234 me   50u  IPv6 0x0      0t0  TCP *:18080 (LISTEN)\n";
        let rows = parse_lsof_occupants(sample, |p| p == 18080);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].pid, 1234);
        assert_eq!(rows[0].port, 18080);
    }

    #[test]
    fn parse_lsof_fuzzy_contains() {
        let sample = b"COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME\njava    1234 me   50u  IPv6 0x0      0t0  TCP *:18080 (LISTEN)\nnode    5678 me   50u  IPv4 0x0      0t0  TCP *:8080 (LISTEN)\nnginx   90 me   50u  IPv4 0x0      0t0  TCP *:80 (LISTEN)\n";
        let rows = parse_lsof_occupants(sample, |p| p.to_string().contains("80"));
        let mut ports: Vec<_> = rows.iter().map(|r| r.port).collect();
        ports.sort_unstable();
        assert_eq!(ports, vec![80, 8080, 18080]);
    }
}
