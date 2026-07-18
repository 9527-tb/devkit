//! Gradle 任务动作。

use crate::models::Project;
use crate::providers::traits::ActionResolver;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct GradleActions;

impl ActionResolver for GradleActions {
    fn kind(&self) -> &'static str {
        "gradle"
    }

    fn resolve_command(
        &self,
        project: &Project,
        action: &str,
    ) -> Result<(String, Vec<String>), String> {
        resolve_command(project, action)
    }
}

/// 将 Gradle 动作解析为 (program, args)。
/// program 默认为 `gradle`；运行时可由 process 层替换为 gradlew。
pub fn resolve_command(project: &Project, action: &str) -> Result<(String, Vec<String>), String> {
    let args: Vec<&str> = match action {
        "clean" => vec!["clean"],
        "build" => vec!["build"],
        "test" => vec!["test"],
        "bootRun" if project.spring_boot => vec!["bootRun"],
        _ => return Err("不支持的 Gradle 操作".into()),
    };
    Ok((
        "gradle".into(),
        args.into_iter().map(str::to_string).collect(),
    ))
}

/// 从 `start` 向上查找 `gradlew` / `gradlew.bat`。
pub fn find_gradle_wrapper(start: &Path) -> Option<PathBuf> {
    #[cfg(windows)]
    let names = ["gradlew.bat", "gradlew.cmd", "gradlew"];
    #[cfg(not(windows))]
    let names = ["gradlew", "gradlew.bat"];

    let mut cursor = Some(start);
    while let Some(dir) = cursor {
        for name in names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        cursor = dir.parent();
    }
    None
}
