//! Node 动作解析（npm/pnpm/yarn/bun）。
//!
//! DONE(rs-provider-node): resolve_command 已从 scan::command_argv 迁入 — DESIGN §6.5

use crate::models::Project;
use crate::providers::traits::ActionResolver;
use std::path::Path;

#[allow(dead_code)]
pub struct NodeActions;

impl ActionResolver for NodeActions {
    fn kind(&self) -> &'static str {
        "node"
    }

    fn resolve_command(
        &self,
        project: &Project,
        action: &str,
    ) -> Result<(String, Vec<String>), String> {
        resolve_command(project, action)
    }
}

/// 默认包管理器占位（实际执行时由设置 `node.packageManager` 覆盖）。
pub fn package_manager(_path: &Path) -> &'static str {
    "npm"
}

#[allow(dead_code)]
pub fn supports_action(action: &str) -> bool {
    action == "install" || action.starts_with("script:")
}

/// 将 Node 动作解析为 (program, args)。
/// program 默认为 npm；`process::run_action` 会按设置替换为 npm/pnpm/yarn。
pub fn resolve_command(project: &Project, action: &str) -> Result<(String, Vec<String>), String> {
    let pm = package_manager(Path::new(&project.path)).to_string();
    if action == "install" {
        return Ok((pm, vec!["install".into()]));
    }
    if let Some(name) = action.strip_prefix("script:") {
        if project.scripts.iter().any(|s| s == name) {
            return Ok((pm, vec!["run".into(), name.to_string()]));
        }
    }
    Err("该项目未定义此操作".into())
}
