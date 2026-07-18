//! Node 依赖树（package.json 静态分组）。
//!
//! DONE(rs-provider-node): 从 deps.rs 迁入封装 — DESIGN §6.5

use crate::deps::node_dependency_tree;
use crate::models::Dependency;
use serde_json::Value;

/// 解析 package.json 依赖树。
pub fn dependency_tree(json: &Value) -> Vec<Dependency> {
    node_dependency_tree(json)
}
