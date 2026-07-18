//! Maven 依赖（pom 解析 / dependency:tree）。
//!
//! DONE(rs-provider-maven): 从 deps.rs 迁入封装 — DESIGN §6.5

use crate::deps::{maven_dependency_tree, parse_pom_tree};
use crate::models::Dependency;
use std::path::Path;

/// 解析 pom.xml 静态依赖树；返回 (deps, spring_boot)。
pub fn parse_pom(project_dir: &Path) -> (Vec<Dependency>, bool) {
    parse_pom_tree(&project_dir.join("pom.xml"))
}

/// 调用 `mvn dependency:tree` 刷新。
pub fn refresh_tree(project_dir: &Path) -> Result<Vec<Dependency>, String> {
    maven_dependency_tree(project_dir)
}
