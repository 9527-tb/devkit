//! Cargo 动作解析。
//!
//! DONE(rs-provider-cargo): build/run/test/check/clean — DESIGN R6

use crate::models::Project;
use crate::providers::traits::ActionResolver;

#[allow(dead_code)]
pub struct CargoActions;

impl ActionResolver for CargoActions {
    fn kind(&self) -> &'static str {
        "cargo"
    }

    fn resolve_command(
        &self,
        project: &Project,
        action: &str,
    ) -> Result<(String, Vec<String>), String> {
        resolve_command(project, action)
    }
}

pub fn resolve_command(_project: &Project, action: &str) -> Result<(String, Vec<String>), String> {
    let args: Vec<&str> = match action {
        "build" => vec!["build"],
        "run" => vec!["run"],
        "test" => vec!["test"],
        "check" => vec!["check"],
        "clean" => vec!["clean"],
        _ => return Err("不支持的 Cargo 操作".into()),
    };
    Ok((
        "cargo".into(),
        args.into_iter().map(str::to_string).collect(),
    ))
}
