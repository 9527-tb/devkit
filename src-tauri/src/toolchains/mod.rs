//! 与 Kind 解耦的本机工具链探测（JDK / Node / Maven Home / …）。
//!
//! TODO(rs-init-progress): 首次初始化按注册表扫描各 toolchain — DESIGN §9.1 / §10.1

#![allow(dead_code)]

pub mod traits;
pub mod jdk;
pub mod node;
pub mod maven_home;

/// 过渡：完整探测仍在 `runtime` 模块，R1 迁入本目录。
#[allow(unused_imports)]
pub use crate::runtime::{detect_all, detect_jdks, detect_maven_home, detect_nodes};
