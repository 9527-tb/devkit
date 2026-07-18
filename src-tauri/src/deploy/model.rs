//! 部署领域模型（规划）。
//!
//! TODO(deploy-engine-skel): Artifact / Environment / DeployProfile 持久化 — DESIGN §9.2

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub target_ids: Vec<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployTarget {
    pub id: String,
    /// ssh | docker | kubernetes | jenkins | …
    pub kind: String,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum DeployRunStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
}
