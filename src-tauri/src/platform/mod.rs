//! 跨平台抽象层：路径、进程、终端、编辑器、Git 等 OS 差异收敛于此。
//! macOS 标题栏红绿灯由 `tauri.macos.conf.json` 的 `trafficLightPosition` 配置（Overlay），不在 Rust 中重定位。

pub mod paths;
pub mod process_kill;
pub mod which;
pub mod net;
pub mod metrics;
pub mod terminal;
pub mod editor;
pub mod external_tools;
pub mod git;

// 供 process / toolchains / settings 逐步迁入时调用；过渡期允许未直接引用
#[allow(unused_imports)]
pub use paths::{config_dir, normalize_path_str, settings_file};
#[allow(unused_imports)]
pub use process_kill::{kill_pid, kill_tree};
#[allow(unused_imports)]
pub use which::{enriched_path, find_executable, find_executable_with_bins};
#[allow(unused_imports)]
pub use net::{list_listening_ports, list_listening_ports_tree};
pub use metrics::{sample_process, ProcessMetrics};
pub use terminal::open_in_terminal;
pub use editor::open_in_editor;
pub use external_tools::detect_external_tools;
pub use git::workspace_git_status;
