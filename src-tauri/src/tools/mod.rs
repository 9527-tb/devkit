//! 开发工具箱子系统。
//!
//! 每个工具独立子模块（`ports` / 未来 `env` / `certs` …），禁止把业务堆进 process/platform。
//! IPC 命令由各工具 `commands` 导出，在 `lib.rs` 注册。

pub mod ports;
pub mod registry;

/// 模块自检占位，确保被 lib 链接。
pub fn init_tools_subsystem() {
    let _ = registry::builtin_tools().len();
}

/// 工具箱公共 IPC（注册表）。
pub mod commands {
    use super::registry::{builtin_tools, ToolMeta};

    #[tauri::command]
    pub fn list_tools() -> Vec<ToolMeta> {
        builtin_tools()
    }
}
