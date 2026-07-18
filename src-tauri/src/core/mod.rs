//! 与具体语言无关的内核：扫描引擎、注册表、领域 DTO、能力描述。
//!
//! 对应 DESIGN.md §6 / §12.3。
//! 禁止在此模块写 `if pom.xml` / `package.json` 等语言分支。
//!
//! TODO(rs-scan-engine): 实现通用扫描引擎 — DESIGN §6.3
//! TODO(rs-registry): Provider 注册表 — DESIGN §6.7
//! TODO(rs-capabilities): Capability 合并 — DESIGN §8.1

pub mod kinds;
pub mod project;
pub mod project_ref;
pub mod caps;
pub mod registry;
pub mod scan_engine;
pub mod errors;

// 过渡期：复用顶层 models，避免破坏现有命令签名
#[allow(unused_imports)]
pub use crate::models::{Dependency, LogEvent, ProcessView, Project};
