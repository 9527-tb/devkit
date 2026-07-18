//! Node 项目 Provider。
//!
//! 对应 DESIGN.md §6.5。
//! DONE(rs-provider-node): detect / actions / deps / toolchain 已落地

pub mod detect;
pub mod actions;
pub mod deps;
pub mod toolchain;

pub use actions::resolve_command;
pub use detect::NodeDetector;

pub const KIND: &str = "node";
