//! Cargo 项目识别（Cargo.toml）。
//!
//! DONE(rs-provider-cargo): enrich 读取 package.name / 默认 actions — DESIGN R6

use crate::models::Project;
use crate::providers::traits::{ProbeResult, ProjectDetector};
use std::fs;
use std::path::Path;

pub struct CargoDetector;

impl ProjectDetector for CargoDetector {
    fn kind(&self) -> &'static str {
        "cargo"
    }

    fn probe(&self, dir: &Path) -> ProbeResult {
        if looks_like_cargo(dir) {
            ProbeResult::Hit
        } else {
            ProbeResult::Miss
        }
    }

    fn enrich(&self, dir: &Path) -> Result<Project, String> {
        cargo_project(dir).ok_or_else(|| "不是有效的 Cargo 项目".into())
    }
}

pub fn looks_like_cargo(dir: &Path) -> bool {
    dir.join("Cargo.toml").is_file()
}

fn parse_package_name(toml: &str) -> Option<String> {
    let mut in_package = false;
    for line in toml.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_package = t == "[package]";
            continue;
        }
        if in_package {
            if let Some(rest) = t.strip_prefix("name") {
                let rest = rest.trim().trim_start_matches('=').trim();
                let name = rest.trim_matches('"').trim_matches('\'').trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

pub fn cargo_project(path: &Path) -> Option<Project> {
    let manifest = path.join("Cargo.toml");
    if !manifest.is_file() {
        return None;
    }
    let text = fs::read_to_string(&manifest).ok()?;
    let name = parse_package_name(&text).unwrap_or_else(|| {
        path.file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("cargo-project")
            .into()
    });
    Some(Project {
        path: path.to_string_lossy().into(),
        name,
        kind: "cargo".into(),
        group: "Cargo".into(),
        spring_boot: false,
        packaging: String::new(),
        scripts: vec![
            "build".into(),
            "run".into(),
            "test".into(),
            "check".into(),
            "clean".into(),
        ],
        dependencies: vec![],
    })
}
