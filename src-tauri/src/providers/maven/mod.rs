//! Maven 项目 Provider。
//!
//! 对应 DESIGN.md §6.5。
//! DONE(rs-provider-maven): detect / actions / deps / toolchain 已落地

pub mod detect;
pub mod actions;
pub mod deps;
pub mod toolchain;

pub use actions::resolve_command;
pub use detect::MavenDetector;

pub const KIND: &str = "maven";
