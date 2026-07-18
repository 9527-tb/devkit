//! Gradle 项目识别（build.gradle* / settings.gradle* + 多模块 include）。

use crate::models::Project;
use crate::providers::traits::{ProbeResult, ProjectDetector};
use std::fs;
use std::path::{Component, Path, PathBuf};

pub struct GradleDetector;

impl ProjectDetector for GradleDetector {
    fn kind(&self) -> &'static str {
        "gradle"
    }

    fn probe(&self, dir: &Path) -> ProbeResult {
        if looks_like_gradle(dir) {
            ProbeResult::Hit
        } else {
            ProbeResult::Miss
        }
    }

    fn enrich(&self, dir: &Path) -> Result<Project, String> {
        gradle_project(dir).ok_or_else(|| "不是有效的 Gradle 项目".into())
    }
}

fn has_build_script(dir: &Path) -> bool {
    dir.join("build.gradle").is_file() || dir.join("build.gradle.kts").is_file()
}

fn settings_path(dir: &Path) -> Option<PathBuf> {
    let groovy = dir.join("settings.gradle");
    if groovy.is_file() {
        return Some(groovy);
    }
    let kts = dir.join("settings.gradle.kts");
    if kts.is_file() {
        return Some(kts);
    }
    None
}

fn has_settings(dir: &Path) -> bool {
    settings_path(dir).is_some()
}

/// 是否为有效 Gradle 工程目录。
///
/// 规则：
/// 1. 存在 build.gradle* 或 settings.gradle*
/// 2. 若有祖先 settings：本目录是 settings 根，或为声明的 include 成员
/// 3. 无 settings 祖先：有 build 清单即可（独立工程）
pub fn looks_like_gradle(dir: &Path) -> bool {
    if !has_build_script(dir) && !has_settings(dir) {
        return false;
    }
    is_gradle_workspace_member(dir)
}

fn is_gradle_workspace_member(dir: &Path) -> bool {
    // 本目录即 settings 根
    if has_settings(dir) {
        return true;
    }
    let mut cursor = dir;
    while let Some(ancestor) = cursor.parent() {
        if let Some(settings) = settings_path(ancestor) {
            let members = read_settings_includes(&settings);
            if members.is_empty() {
                // settings 未声明成员 → 仅根算项目，子目录不算
                return false;
            }
            return members
                .iter()
                .any(|m| module_path_matches(ancestor, m, dir));
        }
        cursor = ancestor;
    }
    // 无 settings 祖先：独立工程需有 build 清单
    has_build_script(dir)
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

/// 将 Gradle include 路径规范为相对目录：`:a:b` / `a:b` → `a/b`。
fn normalize_include_path(raw: &str) -> String {
    let s = raw
        .trim()
        .trim_start_matches(':')
        .trim_start_matches("./")
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string();
    s.replace(':', "/")
}

/// 从一行 `include ...` 中提取项目路径。
fn parse_include_line(line: &str) -> Vec<String> {
    let t = line.trim();
    let rest = if let Some(r) = t.strip_prefix("include") {
        r.trim()
    } else {
        return Vec::new();
    };
    // 跳过 includeBuild / includeFlat 等
    if rest.starts_with("Build") || rest.starts_with("Flat") {
        return Vec::new();
    }

    let mut out = Vec::new();
    let mut chars = rest.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            '"' | '\'' => {
                let quote = c;
                chars.next();
                let mut buf = String::new();
                for ch in chars.by_ref() {
                    if ch == quote {
                        break;
                    }
                    buf.push(ch);
                }
                let norm = normalize_include_path(&buf);
                if !norm.is_empty() {
                    out.push(norm);
                }
            }
            ',' | '(' | ')' | ' ' | '\t' => {
                chars.next();
            }
            _ => {
                chars.next();
            }
        }
    }
    out
}

/// 读取 settings.gradle* 中的 include 成员相对路径。
pub fn read_settings_includes(settings: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(settings) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("include") {
            continue;
        }
        for m in parse_include_line(trimmed) {
            if !out.contains(&m) {
                out.push(m);
            }
        }
    }
    out
}

fn parse_root_project_name(text: &str) -> Option<String> {
    for line in text.lines() {
        let t = line.trim();
        // rootProject.name = "foo" / rootProject.name = 'foo'
        if let Some(rest) = t.strip_prefix("rootProject.name") {
            let rest = rest.trim().trim_start_matches('=').trim();
            let name = rest.trim_matches('"').trim_matches('\'').trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }
    None
}

fn detect_spring_boot(dir: &Path) -> bool {
    for name in ["build.gradle", "build.gradle.kts"] {
        let path = dir.join(name);
        if let Ok(text) = fs::read_to_string(&path) {
            let lower = text.to_ascii_lowercase();
            if lower.contains("org.springframework.boot")
                || lower.contains("spring-boot")
                || lower.contains("id(\"org.springframework.boot\")")
                || lower.contains("id 'org.springframework.boot'")
            {
                return true;
            }
        }
    }
    false
}

fn dir_name(path: &Path) -> String {
    path.file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("gradle-project")
        .into()
}

/// 从目录构造 Gradle Project DTO。
pub fn gradle_project(path: &Path) -> Option<Project> {
    if !looks_like_gradle(path) {
        return None;
    }

    let name = if let Some(settings) = settings_path(path) {
        fs::read_to_string(&settings)
            .ok()
            .and_then(|t| parse_root_project_name(&t))
            .unwrap_or_else(|| dir_name(path))
    } else {
        dir_name(path)
    };

    let spring_boot = detect_spring_boot(path);
    let mut scripts = vec!["clean".into(), "build".into(), "test".into()];
    if spring_boot {
        scripts.push("bootRun".into());
    }

    Some(Project {
        path: path.to_string_lossy().into(),
        name,
        kind: "gradle".into(),
        group: "Gradle".into(),
        spring_boot,
        packaging: String::new(),
        scripts,
        dependencies: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("devkit-gradle-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    #[test]
    fn detects_standalone_build_gradle() {
        let root = temp_dir("standalone");
        write(
            &root.join("build.gradle"),
            "plugins { id 'java' }\n",
        );
        assert!(looks_like_gradle(&root));
        let p = gradle_project(&root).unwrap();
        assert_eq!(p.kind, "gradle");
        assert!(p.scripts.contains(&"build".into()));
        assert!(!p.spring_boot);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn detects_spring_boot_boot_run() {
        let root = temp_dir("boot");
        write(
            &root.join("build.gradle.kts"),
            r#"plugins { id("org.springframework.boot") version "3.2.0" }"#,
        );
        let p = gradle_project(&root).unwrap();
        assert!(p.spring_boot);
        assert!(p.scripts.contains(&"bootRun".into()));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn accepts_only_declared_settings_members() {
        let root = temp_dir("multi");
        write(
            &root.join("settings.gradle"),
            r#"rootProject.name = "demo"
include 'app'
include("lib:core")
"#,
        );
        write(&root.join("build.gradle"), "plugins { id 'java' }\n");
        write(&root.join("app/build.gradle"), "plugins { id 'java' }\n");
        write(
            &root.join("lib/core/build.gradle.kts"),
            "plugins { java }\n",
        );
        write(&root.join("orphan/build.gradle"), "plugins { id 'java' }\n");

        assert!(looks_like_gradle(&root));
        assert!(looks_like_gradle(&root.join("app")));
        assert!(looks_like_gradle(&root.join("lib/core")));
        assert!(!looks_like_gradle(&root.join("orphan")));

        let root_proj = gradle_project(&root).unwrap();
        assert_eq!(root_proj.name, "demo");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn parse_include_paths() {
        assert_eq!(
            parse_include_line(r#"include 'app', 'lib'"#),
            vec!["app".to_string(), "lib".to_string()]
        );
        assert_eq!(
            parse_include_line(r#"include(":modules:core")"#),
            vec!["modules/core".to_string()]
        );
        assert!(parse_include_line("includeBuild('composite')").is_empty());
    }
}
