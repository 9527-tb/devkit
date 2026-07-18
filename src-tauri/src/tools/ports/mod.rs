//! 端口管理工具：查询占用端口的进程并结束进程。
//!
//! 依赖 platform 的进程终止；端口探测逻辑自包含，不与监控 Host 采样混用。

mod service;

/// Tauri IPC（本工具专用命令，前缀 tool_ports_）。
pub mod commands {
    use super::service;

    #[tauri::command]
    pub fn tool_ports_lookup(
        query: String,
        fuzzy: bool,
    ) -> Result<Vec<service::PortOccupant>, String> {
        service::lookup_port(&query, fuzzy)
    }

    #[tauri::command]
    pub fn tool_ports_kill(pid: u32) -> Result<(), String> {
        service::kill_process(pid)
    }
}
