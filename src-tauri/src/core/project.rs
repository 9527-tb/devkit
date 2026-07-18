//! 领域 DTO 重导出与扩展类型占位。
//!
//! TODO(rs-provider-node): ActionSpec 替代仅 scripts: Vec<String> — DESIGN §7.2

#[allow(unused_imports)]
pub use crate::models::*;

/// 可执行动作描述（规划形态；现网仍用 scripts 字符串）。
#[allow(dead_code)]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionSpec {
    pub id: String,
    pub label: String,
    pub group: String,
    pub primary: bool,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
}
