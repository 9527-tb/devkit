//! Maven 生命周期动作。
//!
//! DONE(rs-provider-maven): resolve_command 已从 scan::command_argv 迁入 — DESIGN §6.5

use crate::models::Project;
use crate::providers::traits::ActionResolver;

#[allow(dead_code)]
pub struct MavenActions;

impl ActionResolver for MavenActions {
    fn kind(&self) -> &'static str {
        "maven"
    }

    fn resolve_command(
        &self,
        project: &Project,
        action: &str,
    ) -> Result<(String, Vec<String>), String> {
        resolve_command(project, action)
    }
}

/// 将 Maven 动作解析为 (program, args)。program 固定为 `mvn`，运行时可被 maven_bin 替换。
pub fn resolve_command(project: &Project, action: &str) -> Result<(String, Vec<String>), String> {
    let args: Vec<&str> = match action {
        "clean" => vec!["-B", "clean"],
        "install" => vec!["-B", "install"],
        "clean install" | "clean-install" => vec!["-B", "clean", "install"],
        "package" => vec!["-B", "package"],
        "spring-boot:run" if project.spring_boot => vec!["-B", "spring-boot:run"],
        _ => return Err("不支持的 Maven 操作".into()),
    };
    Ok(("mvn".into(), args.into_iter().map(str::to_string).collect()))
}
