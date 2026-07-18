//! 跨平台进程树终止。
//!
//! DONE(plat-windows): taskkill /T /F — DESIGN §5.2
//! DONE(plat-linux): kill 进程组 — DESIGN §5.2

use std::process::Command;
use std::thread;
use std::time::Duration;

/// 终止指定 PID（单进程；端口管理等场景）。
pub fn kill_pid(pid: u32) -> Result<(), String> {
    if pid == 0 {
        return Err("无效 PID".into());
    }
    #[cfg(windows)]
    {
        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("taskkill 失败: {status}"))
        }
    }
    #[cfg(unix)]
    {
        let pid_s = pid.to_string();
        let _ = Command::new("kill").args(["-TERM", &pid_s]).status();
        thread::sleep(Duration::from_millis(200));
        // 仍存活则强杀
        let alive = unsafe { libc::kill(pid as i32, 0) == 0 };
        if !alive {
            return Ok(());
        }
        let status = Command::new("kill")
            .args(["-KILL", &pid_s])
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() || unsafe { libc::kill(pid as i32, 0) != 0 } {
            Ok(())
        } else {
            Err(format!("kill 失败: {status}"))
        }
    }
}

/// 终止指定 PID 及其子进程。
pub fn kill_tree(pid: u32) -> Result<(), String> {
    #[cfg(windows)]
    {
        let status = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("taskkill 失败: {status}"))
        }
    }
    #[cfg(unix)]
    {
        // 负 PID = 进程组
        let status = Command::new("kill")
            .args(["-TERM", &format!("-{pid}")])
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            return Ok(());
        }
        let status = Command::new("kill")
            .args(["-KILL", &format!("-{pid}")])
            .status()
            .map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("kill 失败: {status}"))
        }
    }
}
