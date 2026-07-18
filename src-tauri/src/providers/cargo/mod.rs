//! Cargo（Rust）项目 Provider — 扩展性验证。
//!
//! DONE(rs-provider-cargo): Cargo.toml 识别与 cargo 动作 — DESIGN R6

pub mod detect;
pub mod actions;

pub use actions::resolve_command;
pub use detect::CargoDetector;

pub const KIND: &str = "cargo";
