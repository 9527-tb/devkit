//! 在系统终端中打开目录。
//!
//! 对应 docs/DEV_PRODUCTIVITY_DESIGN.md ⑥

use std::path::Path;
use std::process::Command;

/// 在 `path` 作为 cwd 打开终端。`terminal` 可选：Terminal / iTerm / warp / wt / cmd 等。
pub fn open_in_terminal(path: &str, terminal: Option<&str>) -> Result<(), String> {
    let dir = Path::new(path);
    if !dir.is_dir() {
        return Err(format!("目录不存在: {path}"));
    }
    let resolved = crate::platform::external_tools::resolve_terminal_id(terminal);
    let pref = Some(resolved.to_ascii_lowercase());

    #[cfg(target_os = "macos")]
    {
        return open_macos(dir, pref.as_deref());
    }
    #[cfg(target_os = "windows")]
    {
        return open_windows(dir, pref.as_deref());
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return open_linux(dir, pref.as_deref());
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (dir, pref);
        Err("当前平台不支持打开终端".into())
    }
}

#[cfg(target_os = "macos")]
fn open_macos(dir: &Path, pref: Option<&str>) -> Result<(), String> {
    let app = match pref {
        Some("iterm") | Some("iterm2") => "iTerm",
        Some("warp") => "Warp",
        Some("terminal") | None => "Terminal",
        Some(other) => other,
    };
    let path_str = dir.to_string_lossy();
    let escaped = path_str.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        r#"tell application "{app}"
  activate
  do script "cd " & quoted form of "{escaped}"
end tell"#
    );
    let status = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .status()
        .map_err(|e| format!("启动终端失败: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        // 回退：open -a
        let status = Command::new("open")
            .args(["-a", app, dir.to_str().unwrap_or(".")])
            .status()
            .map_err(|e| format!("启动终端失败: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("无法打开终端应用 {app}"))
        }
    }
}

#[cfg(target_os = "windows")]
fn open_windows(dir: &Path, pref: Option<&str>) -> Result<(), String> {
    let dir_s = dir.to_string_lossy().to_string();
    match pref {
        Some("cmd") => {
            Command::new("cmd")
                .args(["/C", "start", "cmd", "/K", &format!("cd /d {dir_s}")])
                .spawn()
                .map_err(|e| format!("启动 cmd 失败: {e}"))?;
            Ok(())
        }
        Some("powershell") | Some("pwsh") => {
            let shell = if crate::platform::find_executable("pwsh").is_some() {
                "pwsh"
            } else {
                "powershell"
            };
            Command::new(shell)
                .args([
                    "-NoExit",
                    "-Command",
                    &format!("Set-Location -LiteralPath '{dir_s}'"),
                ])
                .spawn()
                .map_err(|e| format!("启动 PowerShell 失败: {e}"))?;
            Ok(())
        }
        _ => {
            // Windows Terminal 优先
            if crate::platform::find_executable("wt").is_some() {
                Command::new("wt")
                    .args(["-d", &dir_s])
                    .spawn()
                    .map_err(|e| format!("启动 Windows Terminal 失败: {e}"))?;
                return Ok(());
            }
            Command::new("cmd")
                .args(["/C", "start", "cmd", "/K", &format!("cd /d {dir_s}")])
                .spawn()
                .map_err(|e| format!("启动控制台失败: {e}"))?;
            Ok(())
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_linux(dir: &Path, pref: Option<&str>) -> Result<(), String> {
    let candidates: Vec<&str> = match pref {
        Some(p) => vec![p],
        None => vec![
            "x-terminal-emulator",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "alacritty",
            "kitty",
            "xterm",
        ],
    };
    for name in candidates {
        let Some(bin) = crate::platform::find_executable(name) else {
            continue;
        };
        let mut cmd = Command::new(&bin);
        cmd.current_dir(dir);
        // 常见参数：工作目录
        match name {
            "gnome-terminal" => {
                cmd.args(["--working-directory", &dir.to_string_lossy()]);
            }
            "konsole" | "xfce4-terminal" => {
                cmd.args(["--workdir", &dir.to_string_lossy()]);
            }
            "alacritty" | "kitty" => {
                cmd.arg("--working-directory").arg(dir);
            }
            _ => {}
        }
        match cmd.spawn() {
            Ok(_) => return Ok(()),
            Err(_) => continue,
        }
    }
    Err("未找到可用的终端模拟器".into())
}
