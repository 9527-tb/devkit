//! Gradle 项目 Provider。
//!
//! DONE(rs-provider-gradle): build.gradle* / settings.gradle* 识别与常用任务

pub mod detect;
pub mod actions;

pub use actions::resolve_command;
pub use detect::GradleDetector;

pub const KIND: &str = "gradle";
