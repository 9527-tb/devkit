//! 用户配置加载 / 保存 / 迁移 / 首次初始化。
//!
//! 对应 DESIGN.md §7.6。完整逻辑暂由 `runtime` 提供，R1 拆入子模块。
//!
//! TODO(fe-settings-store): 前端对接本模块 IPC — DESIGN R1
//! TODO(rs-init-progress): init 进度事件 — DESIGN §4.2

pub mod schema;
pub mod load_save;
pub mod migrate;
pub mod init;

#[allow(unused_imports)]
pub use crate::runtime::{
    initialize_runtime_settings, load_settings, save_settings, settings_file_exists,
    DetectedRuntimes, RuntimeEntry, RuntimeSettings,
};
