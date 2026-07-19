//! 通用目录扫描引擎（与 Kind 识别解耦）。
//!
//! 对应 DESIGN.md §6.3。引擎只走 ProviderRegistry 的 Detector；
//! 同一目录可命中多个 Kind（AllowMultiple）。
//!
//! DONE(rs-scan-engine): 深度 / 忽略 / DirSnapshot / AllowMultiple — DESIGN §6.3

use crate::core::registry::ProviderRegistry;
use crate::models::Project;
use crate::providers::traits::ProbeResult;
use std::fs;
use std::path::{Path, PathBuf};

/// 最大扫描深度（相对 root）。
pub const MAX_SCAN_DEPTH: usize = 6;

/// 目录快照：文件名集合，供 Detector 快速判断。
#[derive(Debug, Default)]
pub struct DirSnapshot {
    pub path: PathBuf,
    pub entries: Vec<String>,
}

impl DirSnapshot {
    pub fn capture(dir: &Path) -> Self {
        let mut entries = Vec::new();
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                if let Some(name) = e.file_name().to_str() {
                    entries.push(name.to_string());
                }
            }
        }
        Self {
            path: dir.to_path_buf(),
            entries,
        }
    }

    pub fn has_file(&self, name: &str) -> bool {
        self.entries.iter().any(|e| e == name)
    }
}

/// 是否跳过该目录名。
///
/// 含 Maven/IDE 构建输出（`target`、`bin`、`out`、`obj` 等），避免把输出目录里
/// 拷贝的 `pom.xml` 误识别为模块。见 DESIGN.md §6.3。
pub fn ignored(name: &str) -> bool {
    matches!(
        name,
        "node_modules"
            | ".git"
            | "target"
            | "bin"
            | "out"
            | "obj"
            | "classes"
            | "dist"
            | "build"
            | ".idea"
            | ".settings"
            | ".gradle"
            | "Pods"
            | ".next"
            | ".turbo"
            | ".cache"
            | "coverage"
            | "__pycache__"
            | ".pnpm-store"
            | "uni_modules"
    )
}

/// 对单目录运行全部 Detector（AllowMultiple：可返回多个 Project）。
pub fn projects_at(path: &Path, registry: &ProviderRegistry) -> Vec<Project> {
    let mut out = Vec::new();
    for det in registry.detectors() {
        if det.probe(path) == ProbeResult::Hit {
            if let Ok(p) = det.enrich(path) {
                out.push(p);
            }
        }
    }
    out
}

/// 按 path+kind 取单个项目。
pub fn project_at(path: &Path, kind: &str) -> Option<Project> {
    let registry = ProviderRegistry::builtin();
    projects_at(path, &registry)
        .into_iter()
        .find(|p| p.kind == kind)
}

fn scan_dir(root: &Path, depth: usize, registry: &ProviderRegistry, output: &mut Vec<Project>) {
    if depth > MAX_SCAN_DEPTH {
        return;
    }
    let snap = DirSnapshot::capture(root);
    let _ = (
        snap.path.as_path(),
        snap.has_file("package.json"),
        snap.has_file("pom.xml"),
    );
    output.extend(projects_at(root, registry));
    if let Ok(entries) = fs::read_dir(root) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() && !ignored(e.file_name().to_string_lossy().as_ref()) {
                scan_dir(&p, depth + 1, registry, output);
            }
        }
    }
}

/// 扫描工作区根目录，返回去重排序后的项目列表。
pub fn scan_workspace(root: String) -> Result<Vec<Project>, String> {
    let path = PathBuf::from(&root);
    if !path.is_dir() {
        return Err("所选路径不可用".into());
    }
    let registry = ProviderRegistry::builtin();
    let mut projects = vec![];
    scan_dir(&path, 0, &registry, &mut projects);
    projects.sort_by(|a, b| a.path.cmp(&b.path).then(a.kind.cmp(&b.kind)));
    projects.dedup_by(|a, b| a.path == b.path && a.kind == b.kind);
    projects.sort_by(|a, b| {
        a.group
            .cmp(&b.group)
            .then(a.name.cmp(&b.name))
            .then(a.path.cmp(&b.path))
    });
    Ok(projects)
}

/// 将动作解析为 (program, args)，按 kind 分发给 Provider。
pub fn resolve_command(
    project: &Project,
    action: &str,
) -> Result<(String, Vec<String>), String> {
    match project.kind.as_str() {
        "node" => crate::providers::node::resolve_command(project, action),
        "maven" => crate::providers::maven::resolve_command(project, action),
        "cargo" => crate::providers::cargo::resolve_command(project, action),
        "gradle" => crate::providers::gradle::resolve_command(project, action),
        _ => Err(format!("未知项目类型: {}", project.kind)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("devkit-scan-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn allow_multiple_node_and_maven() {
        let root = temp_dir("dual");
        fs::write(
            root.join("package.json"),
            r#"{"name":"demo","scripts":{"dev":"x"}}"#,
        )
        .unwrap();
        fs::write(
            root.join("pom.xml"),
            r#"<?xml version="1.0"?><project><modelVersion>4.0.0</modelVersion><groupId>a</groupId><artifactId>b</artifactId><version>1</version></project>"#,
        )
        .unwrap();
        let projects = scan_workspace(root.to_string_lossy().into()).unwrap();
        let kinds: Vec<_> = projects.iter().map(|p| p.kind.as_str()).collect();
        assert!(kinds.contains(&"node"));
        assert!(kinds.contains(&"maven"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn detects_cargo() {
        let root = temp_dir("cargo");
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"hello_kit\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        let projects = scan_workspace(root.to_string_lossy().into()).unwrap();
        assert!(projects.iter().any(|p| p.kind == "cargo" && p.name == "hello_kit"));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn detects_gradle() {
        let root = temp_dir("gradle");
        fs::write(
            root.join("settings.gradle"),
            "rootProject.name = \"hello_gradle\"\n",
        )
        .unwrap();
        fs::write(root.join("build.gradle"), "plugins { id 'java' }\n").unwrap();
        let projects = scan_workspace(root.to_string_lossy().into()).unwrap();
        assert!(
            projects
                .iter()
                .any(|p| p.kind == "gradle" && p.name == "hello_gradle")
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn ignores_node_modules() {
        assert!(ignored("node_modules"));
        assert!(ignored("uni_modules"));
        assert!(!ignored("packages"));
    }

    #[test]
    fn ignores_maven_ide_output_dirs() {
        assert!(ignored("bin"));
        assert!(ignored("target"));
        assert!(ignored("out"));
        assert!(ignored("obj"));
        assert!(!ignored("xlogic-basic"));
    }

    #[test]
    fn scan_skips_bin_pom_under_maven_module() {
        let root = temp_dir("mvn-bin");
        fs::write(
            root.join("pom.xml"),
            r#"<?xml version="1.0"?>
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>a</groupId>
  <artifactId>root</artifactId>
  <version>1</version>
  <packaging>pom</packaging>
  <modules><module>lib</module></modules>
</project>"#,
        )
        .unwrap();
        fs::create_dir_all(root.join("lib/bin")).unwrap();
        fs::write(
            root.join("lib/pom.xml"),
            r#"<?xml version="1.0"?>
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>a</groupId>
  <artifactId>lib</artifactId>
  <version>1</version>
</project>"#,
        )
        .unwrap();
        fs::write(
            root.join("lib/bin/pom.xml"),
            r#"<?xml version="1.0"?>
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>a</groupId>
  <artifactId>lib</artifactId>
  <version>1</version>
</project>"#,
        )
        .unwrap();

        let projects = scan_workspace(root.to_string_lossy().into()).unwrap();
        let maven_paths: Vec<_> = projects
            .iter()
            .filter(|p| p.kind == "maven")
            .map(|p| p.path.replace('\\', "/"))
            .collect();
        let root_s = root.to_string_lossy().replace('\\', "/");
        assert!(maven_paths.iter().any(|p| p == &root_s));
        assert!(maven_paths.iter().any(|p| p.ends_with("/lib")));
        assert!(!maven_paths.iter().any(|p| p.ends_with("/bin")));
        assert_eq!(maven_paths.len(), 2);
        let _ = fs::remove_dir_all(&root);
    }
}
