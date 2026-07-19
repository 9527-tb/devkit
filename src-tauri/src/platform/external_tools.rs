//! 探测本机可用的编辑器 / 控制台，供设置页选择与打开时回退。

use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExternalToolOption {
    pub id: String,
    pub label: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DetectedExternalTools {
    pub editors: Vec<ExternalToolOption>,
    pub terminals: Vec<ExternalToolOption>,
    pub default_editor: String,
    pub default_terminal: String,
}

pub fn detect_external_tools() -> DetectedExternalTools {
    let editors = detect_editors();
    let terminals = detect_terminals();
    let default_editor = editors
        .first()
        .map(|e| e.id.clone())
        .unwrap_or_else(|| "cursor".into());
    let default_terminal = terminals
        .first()
        .map(|t| t.id.clone())
        .unwrap_or_else(|| default_terminal_id().into());
    DetectedExternalTools {
        editors,
        terminals,
        default_editor,
        default_terminal,
    }
}

pub fn resolve_editor_id(configured: Option<&str>) -> String {
    let trimmed = configured.map(str::trim).filter(|s| !s.is_empty());
    if let Some(id) = trimmed {
        return id.to_string();
    }
    detect_external_tools().default_editor
}

pub fn resolve_terminal_id(configured: Option<&str>) -> String {
    let trimmed = configured.map(str::trim).filter(|s| !s.is_empty());
    if let Some(id) = trimmed {
        return id.to_string();
    }
    detect_external_tools().default_terminal
}

fn opt(id: &str, label: &str) -> ExternalToolOption {
    ExternalToolOption {
        id: id.into(),
        label: label.into(),
    }
}

fn editor_available(id: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        let apps: &[&str] = match id {
            "cursor" => &["Cursor"],
            "code" => &["Visual Studio Code", "VSCode", "Code"],
            "idea" => &[
                "IntelliJ IDEA",
                "IntelliJ IDEA CE",
                "IntelliJ IDEA Ultimate",
            ],
            "subl" => &["Sublime Text"],
            _ => &[],
        };
        if apps.iter().any(|name| macos_app_exists(name)) {
            return true;
        }
    }

    if let Some(bin) = crate::platform::find_executable(id) {
        let text = bin.to_string_lossy();
        // Cursor 常劫持 PATH 里的 `code`，不能据此认定已装 VS Code
        if id == "code" && text.contains("Cursor.app") {
            return false;
        }
        return true;
    }
    false
}

fn detect_editors() -> Vec<ExternalToolOption> {
    // 优先级：先装的常见开发工具靠前，作为默认
    let candidates = [
        ("cursor", "Cursor"),
        ("code", "VS Code"),
        ("idea", "IntelliJ IDEA"),
        ("subl", "Sublime Text"),
    ];
    candidates
        .into_iter()
        .filter(|(id, _)| editor_available(id))
        .map(|(id, label)| opt(id, label))
        .collect()
}

fn detect_terminals() -> Vec<ExternalToolOption> {
    #[cfg(target_os = "macos")]
    {
        let mut out = Vec::new();
        // 系统默认控制台优先
        if macos_app_exists("Terminal") {
            out.push(opt("terminal", "Terminal"));
        }
        if macos_app_exists("iTerm") || macos_app_exists("iTerm2") {
            out.push(opt("iterm", "iTerm"));
        }
        if macos_app_exists("Warp") {
            out.push(opt("warp", "Warp"));
        }
        if out.is_empty() {
            out.push(opt("terminal", "Terminal"));
        }
        return out;
    }
    #[cfg(target_os = "windows")]
    {
        let mut out = Vec::new();
        if crate::platform::find_executable("wt").is_some() {
            out.push(opt("wt", "Windows Terminal"));
        }
        out.push(opt("cmd", "Command Prompt"));
        if crate::platform::find_executable("powershell").is_some()
            || crate::platform::find_executable("pwsh").is_some()
        {
            out.push(opt("powershell", "PowerShell"));
        }
        return out;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let candidates = [
            ("x-terminal-emulator", "Default terminal"),
            ("gnome-terminal", "GNOME Terminal"),
            ("konsole", "Konsole"),
            ("xfce4-terminal", "XFCE Terminal"),
            ("alacritty", "Alacritty"),
            ("kitty", "Kitty"),
            ("xterm", "XTerm"),
        ];
        let mut out: Vec<_> = candidates
            .into_iter()
            .filter(|(id, _)| crate::platform::find_executable(id).is_some())
            .map(|(id, label)| opt(id, label))
            .collect();
        if out.is_empty() {
            out.push(opt("x-terminal-emulator", "Default terminal"));
        }
        return out;
    }
    #[cfg(not(any(unix, windows)))]
    {
        vec![opt("terminal", "Terminal")]
    }
}

fn default_terminal_id() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "terminal"
    }
    #[cfg(target_os = "windows")]
    {
        "wt"
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        "x-terminal-emulator"
    }
    #[cfg(not(any(unix, windows)))]
    {
        "terminal"
    }
}

#[cfg(target_os = "macos")]
pub fn macos_app_exists(name: &str) -> bool {
    use std::path::Path;
    let app = format!("{name}.app");
    Path::new("/Applications").join(&app).is_dir()
        || dirs_home()
            .map(|h| h.join("Applications").join(&app).is_dir())
            .unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn dirs_home() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(std::path::PathBuf::from)
}
