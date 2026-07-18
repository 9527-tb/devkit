//! Node 项目识别（package.json）。
//!
//! DONE(rs-provider-node): detect/enrich 已从 scan.rs 迁入 — DESIGN §6.5

use crate::models::Project;
use crate::providers::node::deps;
use crate::providers::traits::{ProbeResult, ProjectDetector};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub struct NodeDetector;

impl ProjectDetector for NodeDetector {
    fn kind(&self) -> &'static str {
        "node"
    }

    fn probe(&self, dir: &Path) -> ProbeResult {
        if looks_like_node(dir) {
            ProbeResult::Hit
        } else {
            ProbeResult::Miss
        }
    }

    fn enrich(&self, dir: &Path) -> Result<Project, String> {
        node_project(dir).ok_or_else(|| "不是有效的 Node 项目".into())
    }
}

pub fn looks_like_node(dir: &Path) -> bool {
    dir.join("package.json").is_file()
}

fn read_json(path: &Path) -> Option<Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|v| serde_json::from_str(&v).ok())
}

/// 从目录构造 Node Project DTO。
pub fn node_project(path: &Path) -> Option<Project> {
    let package = path.join("package.json");
    if !package.is_file() {
        return None;
    }
    let json = read_json(&package)?;
    let mut scripts = json
        .get("scripts")
        .and_then(|v| v.as_object())
        .map(|o| o.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    scripts.sort();
    let name = json
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            path.file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("node-project")
                .into()
        });
    Some(Project {
        path: path.to_string_lossy().into(),
        name,
        kind: "node".into(),
        group: "Node".into(),
        spring_boot: false,
        packaging: String::new(),
        scripts,
        dependencies: deps::dependency_tree(&json),
    })
}
