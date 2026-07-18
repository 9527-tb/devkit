//! Maven 项目识别（pom.xml + 反应堆 modules 规则）。
//!
//! DONE(rs-provider-maven): detect/enrich 已从 scan.rs 迁入 — DESIGN §6.5

use crate::models::Project;
use crate::providers::maven::deps;
use crate::providers::traits::{ProbeResult, ProjectDetector};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub struct MavenDetector;

impl ProjectDetector for MavenDetector {
    fn kind(&self) -> &'static str {
        "maven"
    }

    fn probe(&self, dir: &Path) -> ProbeResult {
        if looks_like_maven(dir) {
            ProbeResult::Hit
        } else {
            ProbeResult::Miss
        }
    }

    fn enrich(&self, dir: &Path) -> Result<Project, String> {
        maven_project(dir).ok_or_else(|| "不是有效的 Maven 项目".into())
    }
}

/// 是否为有效 Maven 模块目录。
///
/// 规则：
/// 1. 目录下存在 `pom.xml`
/// 2. 目录名不是构建/IDE 输出目录（`bin` / `target` 等由扫描忽略；此处再兜底）
/// 3. 若父目录有 `pom.xml`：
///    - 父 POM 声明了 `<modules>` → 仅当本目录是其中某个 module 路径时才算
///    - 父 POM 无 `<modules>`（叶子构件）→ 其下嵌套 pom（如 Eclipse `bin/pom.xml`）不算模块
pub fn looks_like_maven(dir: &Path) -> bool {
    if !dir.join("pom.xml").is_file() {
        return false;
    }
    if is_build_output_dir_name(dir) {
        return false;
    }
    is_maven_reactor_member(dir)
}

fn is_build_output_dir_name(dir: &Path) -> bool {
    matches!(
        dir.file_name().and_then(|s| s.to_str()).unwrap_or(""),
        "bin" | "target" | "out" | "classes" | "generated-sources" | "generated-test-sources"
    )
}

/// 按最近祖先 POM 的 `<module>` 声明判断是否为反应堆成员。
///
/// 支持 `<module>bridge/core</module>` 这类多级相对路径：中间目录可能没有 pom。
fn is_maven_reactor_member(dir: &Path) -> bool {
    let mut cursor = dir;
    while let Some(ancestor) = cursor.parent() {
        let ancestor_pom = ancestor.join("pom.xml");
        if ancestor_pom.is_file() {
            let modules = read_pom_modules(&ancestor_pom);
            if modules.is_empty() {
                // 最近 Maven 祖先是叶子构件 → 其下嵌套 pom 不是模块
                return false;
            }
            return modules
                .iter()
                .any(|m| module_path_matches(ancestor, m, dir));
        }
        cursor = ancestor;
    }
    // 无 Maven 祖先：独立工程
    true
}

fn module_path_matches(parent: &Path, module: &str, dir: &Path) -> bool {
    let candidate = parent.join(module);
    path_eq_normalized(&candidate, dir)
}

fn path_eq_normalized(a: &Path, b: &Path) -> bool {
    normalize_path(a) == normalize_path(b)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// 读取 POM 中的 `<module>` 相对路径列表。
pub fn read_pom_modules(pom: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(pom) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut rest = text.as_str();
    while let Some(start) = rest.find("<module>") {
        rest = &rest[start + "<module>".len()..];
        let Some(end) = rest.find("</module>") else {
            break;
        };
        let raw = rest[..end].trim();
        rest = &rest[end + "</module>".len()..];
        if raw.contains('<') {
            continue;
        }
        let norm = raw
            .trim_start_matches("./")
            .replace('\\', "/")
            .trim_end_matches('/')
            .to_string();
        if !norm.is_empty() {
            out.push(norm);
        }
    }
    out
}

/// 读取 POM `<packaging>`；未声明时 Maven 默认为 jar。
pub fn read_pom_packaging(pom: &Path) -> String {
    let Ok(text) = fs::read_to_string(pom) else {
        return "jar".into();
    };
    if let Some(start) = text.find("<packaging>") {
        let rest = &text[start + "<packaging>".len()..];
        if let Some(end) = rest.find("</packaging>") {
            let raw = rest[..end].trim();
            if !raw.is_empty() && !raw.contains('<') {
                return raw.to_ascii_lowercase();
            }
        }
    }
    "jar".into()
}

/// 从目录构造 Maven Project DTO。
pub fn maven_project(path: &Path) -> Option<Project> {
    if !looks_like_maven(path) {
        return None;
    }
    let name = path
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("maven-project")
        .into();
    let (dependencies, spring_boot) = deps::parse_pom(path);
    let packaging = read_pom_packaging(&path.join("pom.xml"));
    let mut scripts = vec![
        "clean".into(),
        "install".into(),
        "clean install".into(),
        "package".into(),
    ];
    if spring_boot {
        scripts.push("spring-boot:run".into());
    }
    Some(Project {
        path: path.to_string_lossy().into(),
        name,
        kind: "maven".into(),
        group: "Maven".into(),
        spring_boot,
        packaging,
        scripts,
        dependencies,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("devkit-mvn-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_pom(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    #[test]
    fn rejects_eclipse_bin_copy_under_leaf_module() {
        let root = temp_dir("bin-copy");
        write_pom(
            &root.join("pom.xml"),
            r#"<?xml version="1.0"?>
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>a</groupId>
  <artifactId>leaf</artifactId>
  <version>1</version>
</project>"#,
        );
        write_pom(
            &root.join("bin/pom.xml"),
            r#"<?xml version="1.0"?>
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>a</groupId>
  <artifactId>leaf</artifactId>
  <version>1</version>
</project>"#,
        );
        assert!(looks_like_maven(&root));
        assert!(!looks_like_maven(&root.join("bin")));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn accepts_only_declared_reactor_modules() {
        let root = temp_dir("reactor");
        write_pom(
            &root.join("pom.xml"),
            r#"<?xml version="1.0"?>
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>a</groupId>
  <artifactId>root</artifactId>
  <version>1</version>
  <packaging>pom</packaging>
  <modules>
    <module>xlogic-basic</module>
    <module>bridge/core</module>
  </modules>
</project>"#,
        );
        write_pom(
            &root.join("xlogic-basic/pom.xml"),
            r#"<?xml version="1.0"?>
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>a</groupId>
  <artifactId>xlogic-basic</artifactId>
  <version>1</version>
</project>"#,
        );
        write_pom(
            &root.join("bridge/core/pom.xml"),
            r#"<?xml version="1.0"?>
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>a</groupId>
  <artifactId>core</artifactId>
  <version>1</version>
</project>"#,
        );
        write_pom(
            &root.join("orphan/pom.xml"),
            r#"<?xml version="1.0"?>
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>a</groupId>
  <artifactId>orphan</artifactId>
  <version>1</version>
</project>"#,
        );

        write_pom(
            &root.join("bridge/junk/pom.xml"),
            r#"<?xml version="1.0"?>
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>a</groupId>
  <artifactId>junk</artifactId>
  <version>1</version>
</project>"#,
        );

        assert!(looks_like_maven(&root));
        assert!(looks_like_maven(&root.join("xlogic-basic")));
        assert!(looks_like_maven(&root.join("bridge/core")));
        assert!(!looks_like_maven(&root.join("orphan")));
        assert!(!looks_like_maven(&root.join("bridge/junk")));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn read_pom_packaging_defaults_to_jar() {
        let root = temp_dir("pkg-default");
        let pom = root.join("pom.xml");
        write_pom(
            &pom,
            r#"<?xml version="1.0"?>
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>a</groupId>
  <artifactId>leaf</artifactId>
  <version>1</version>
</project>"#,
        );
        assert_eq!(read_pom_packaging(&pom), "jar");
        write_pom(
            &pom,
            r#"<?xml version="1.0"?>
<project>
  <modelVersion>4.0.0</modelVersion>
  <groupId>a</groupId>
  <artifactId>parent</artifactId>
  <version>1</version>
  <packaging>pom</packaging>
</project>"#,
        );
        assert_eq!(read_pom_packaging(&pom), "pom");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn read_pom_modules_strips_dot_slash() {
        let root = temp_dir("mods");
        let pom = root.join("pom.xml");
        write_pom(
            &pom,
            r#"<modules>
  <module>./foo</module>
  <module>bar/</module>
</modules>"#,
        );
        let mods = read_pom_modules(&pom);
        assert_eq!(mods, vec!["foo".to_string(), "bar".to_string()]);
        let _ = fs::remove_dir_all(&root);
    }
}
