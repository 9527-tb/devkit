//! 工作区可分享配置：`<root>/.devkit/workspace.json`
//!
//! 对应 docs/DEV_PRODUCTIVITY_DESIGN.md ⑫

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub const WORKSPACE_CONFIG_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceConfig {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_filter: Option<Value>,
    /// 运行计划（规范字段）；读时兼容旧 `pipelines`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_plans: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipelines: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probes: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended: Option<Value>,
}

fn default_version() -> u32 {
    WORKSPACE_CONFIG_VERSION
}

pub fn workspace_config_path(root: &str) -> std::path::PathBuf {
    Path::new(root).join(".devkit").join("workspace.json")
}

pub fn load_workspace_config(root: String) -> Result<Option<WorkspaceConfig>, String> {
    let path = workspace_config_path(&root);
    if !path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).map_err(|e| format!("读取 workspace.json 失败: {e}"))?;
    let mut config: WorkspaceConfig =
        serde_json::from_str(&text).map_err(|e| format!("解析 workspace.json 失败: {e}"))?;
    sanitize_config(&mut config);
    // 兼容：旧 pipelines → runPlans
    if config.run_plans.is_none() {
        if let Some(p) = config.pipelines.clone() {
            config.run_plans = Some(p);
        }
    }
    Ok(Some(config))
}

pub fn save_workspace_config(root: String, config: WorkspaceConfig) -> Result<(), String> {
    let root_path = Path::new(&root);
    if !root_path.is_dir() {
        return Err(format!("工作区目录不存在: {root}"));
    }
    let mut config = config;
    config.version = WORKSPACE_CONFIG_VERSION;
    sanitize_config(&mut config);
    let dir = root_path.join(".devkit");
    fs::create_dir_all(&dir).map_err(|e| format!("创建 .devkit 失败: {e}"))?;
    let path = dir.join("workspace.json");
    let text =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化 workspace.json 失败: {e}"))?;
    fs::write(&path, text).map_err(|e| format!("写入 workspace.json 失败: {e}"))
}

/// 禁止写入本机工具链绝对路径等敏感字段。
fn sanitize_config(config: &mut WorkspaceConfig) {
    strip_toolchain_paths(&mut config.project_filter);
    strip_toolchain_paths(&mut config.run_plans);
    strip_toolchain_paths(&mut config.pipelines);
    strip_toolchain_paths(&mut config.probes);
    if let Some(rec) = config.recommended.as_mut() {
        strip_toolchain_paths_value(rec);
    }
}

fn strip_toolchain_paths(value: &mut Option<Value>) {
    let Some(v) = value else {
        return;
    };
    strip_toolchain_paths_value(v);
}

fn strip_toolchain_paths_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for key in [
                "jdks",
                "nodes",
                "mavenHome",
                "maven_home",
                "java",
                "node",
                "jdk",
                "JAVA_HOME",
                "NODE_HOME",
            ] {
                map.remove(key);
            }
            for v in map.values_mut() {
                strip_toolchain_paths_value(v);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                strip_toolchain_paths_value(v);
            }
        }
        _ => {}
    }
}
