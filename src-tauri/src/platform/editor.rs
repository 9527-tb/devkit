//! 在外部编辑器中打开路径。
//!
//! 对应 docs/DEV_PRODUCTIVITY_DESIGN.md ⑤
//!
//! 注意：Cursor 常把 `/usr/local/bin/code` 指到自身，因此 macOS 必须优先按 App 打开，
//! 不能只信 PATH 里的 `code`。

use std::path::{Path, PathBuf};
use std::process::Command;

/// 用配置的编辑器 CLI / App 打开 `path`。
/// `editor` 示例：`cursor` / `code` / `idea` / 自定义含 `{path}` 的模板。
pub fn open_in_editor(path: &str, editor: Option<&str>) -> Result<(), String> {
    let target = Path::new(path);
    if !target.exists() {
        return Err(format!("路径不存在: {path}"));
    }

    let resolved = crate::platform::external_tools::resolve_editor_id(editor);
    let raw = resolved.as_str();

    if raw.contains("{path}") {
        return run_template(raw, path);
    }

    let lower = raw.to_ascii_lowercase();
    let cli = match lower.as_str() {
        "cursor" => "cursor",
        "code" | "vscode" | "vs code" | "vs-code" => "code",
        "idea" | "intellij" => "idea",
        "subl" | "sublime" => "subl",
        other => other,
    };

    // macOS：优先按 .app 打开，避开被 Cursor 劫持的 `code` CLI
    #[cfg(target_os = "macos")]
    {
        if let Some(app) = first_macos_editor_app(cli) {
            let status = Command::new("open")
                .args(["-a", app, path])
                .status()
                .map_err(|e| format!("open -a 失败: {e}"))?;
            if status.success() {
                return Ok(());
            }
        }
    }

    if let Some(bin) = resolve_editor_cli(cli) {
        Command::new(&bin)
            .arg(path)
            .spawn()
            .map_err(|e| format!("启动编辑器失败 ({cli}): {e}"))?;
        return Ok(());
    }

    Err(format!(
        "未找到编辑器 `{cli}`。请在设置中配置「默认编辑器」，或安装对应应用 / CLI。"
    ))
}

/// 解析真实 CLI；若 `code` 实际落在 Cursor.app 内则跳过（避免误开 Cursor）。
fn resolve_editor_cli(cli: &str) -> Option<PathBuf> {
    let bin = crate::platform::find_executable(cli)?;
    let text = bin.to_string_lossy();
    if cli == "code" && text.contains("Cursor.app") {
        return None;
    }
    if cli == "cursor" && text.contains("Visual Studio Code.app") {
        return None;
    }
    Some(bin)
}

#[cfg(target_os = "macos")]
fn first_macos_editor_app(cli: &str) -> Option<&'static str> {
    macos_editor_app_names(cli)
        .iter()
        .copied()
        .find(|name| crate::platform::external_tools::macos_app_exists(name))
}

#[cfg(target_os = "macos")]
fn macos_editor_app_names(cli: &str) -> &'static [&'static str] {
    match cli {
        "cursor" => &["Cursor"],
        // 本机常见安装名是 VSCode.app；官方包为 Visual Studio Code.app
        "code" => &["Visual Studio Code", "VSCode", "Code"],
        "idea" => &[
            "IntelliJ IDEA",
            "IntelliJ IDEA CE",
            "IntelliJ IDEA Ultimate",
        ],
        "subl" => &["Sublime Text"],
        _ => &[],
    }
}

fn run_template(template: &str, path: &str) -> Result<(), String> {
    let cmdline = template.replace("{path}", path);
    let parts: Vec<&str> = cmdline.split_whitespace().collect();
    let Some((program, args)) = parts.split_first() else {
        return Err("编辑器命令模板为空".into());
    };
    let bin = crate::platform::find_executable(program)
        .unwrap_or_else(|| Path::new(program).to_path_buf());
    Command::new(&bin)
        .args(args)
        .spawn()
        .map_err(|e| format!("执行编辑器命令失败: {e}"))?;
    Ok(())
}
