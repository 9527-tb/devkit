//! 跨平台抽象层。
//!
//! 对应 DESIGN.md §5：路径、进程终止、which、端口列举等 OS 差异收敛于此。
//! 业务模块（providers / process）只依赖本模块公开 API，禁止写死平台路径。
//!
//! DONE(plat-windows): 路径 / taskkill / netstat — DESIGN §5.2
//! DONE(plat-linux): kill 进程组 / lsof — DESIGN §5.2

pub mod paths;
pub mod process_kill;
pub mod which;
pub mod net;
pub mod metrics;
pub mod terminal;
pub mod editor;
pub mod external_tools;
pub mod git;
#[cfg(target_os = "macos")]
pub mod macos_titlebar;

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
