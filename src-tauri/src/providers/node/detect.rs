//! Node 项目识别（有效 package.json）。
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

/// 目录是否为可运行的 Node 项目根（非 uni_modules 插件、非仅有 id 的组件 manifest）。
pub fn looks_like_node(dir: &Path) -> bool {
    parse_node_package(&dir.join("package.json")).is_some()
}

fn read_json(path: &Path) -> Option<Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|v| serde_json::from_str(&v).ok())
}

/// 解析并校验 package.json；无效则 None。
fn parse_node_package(path: &Path) -> Option<Value> {
    if !path.is_file() {
        return None;
    }
    let json = read_json(path)?;
    // uni-app 插件市场组件（通常在 uni_modules 内）
    if json.get("dcloudext").is_some() {
        return None;
    }
    if json
        .get("uni_modules")
        .and_then(|v| v.get("platforms"))
        .is_some()
    {
        return None;
    }
    // 常规 npm 项目必须有非空 name
    if !json
        .get("name")
        .and_then(|v| v.as_str())
        .is_some_and(|s| !s.trim().is_empty())
    {
        return None;
    }
    Some(json)
}

/// 从目录构造 Node Project DTO。
pub fn node_project(path: &Path) -> Option<Project> {
    let json = parse_node_package(&path.join("package.json"))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("devkit-node-detect-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn accepts_valid_npm_project() {
        let dir = temp_dir("valid");
        fs::write(
            dir.join("package.json"),
            r#"{"name":"demo","scripts":{"start":"vite"}}"#,
        )
        .unwrap();
        assert!(looks_like_node(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_uni_module_manifest() {
        let dir = temp_dir("uni-mod");
        fs::write(
            dir.join("package.json"),
            r#"{"id":"mp-html","dcloudext":{},"uni_modules":{"platforms":{}}}"#,
        )
        .unwrap();
        assert!(!looks_like_node(&dir));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_package_without_name() {
        let dir = temp_dir("noname");
        fs::write(dir.join("package.json"), r#"{"version":"1.0.0"}"#).unwrap();
        assert!(!looks_like_node(&dir));
        let _ = fs::remove_dir_all(&dir);
    }
}
