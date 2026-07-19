use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub key: String,
    pub name: String,
    pub version: String,
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<Dependency>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub path: String,
    pub name: String,
    pub kind: String,
    pub group: String,
    pub spring_boot: bool,
    /// Maven `<packaging>`；缺省为 jar。非 Maven 为空串。
    #[serde(default)]
    pub packaging: String,
    pub scripts: Vec<String>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProcessView {
    pub pid: u32,
    pub action: String,
    pub port: Option<String>,
    pub kind: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub path: String,
    pub kind: String,
    pub pid: u32,
    pub line: String,
}

/// 一次托管运行结束后的结构化摘要。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunSummary {
    pub project_key: String,
    pub path: String,
    pub kind: String,
    pub pid: u32,
    pub action: String,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
    pub error_lines: Vec<String>,
    pub hints: Vec<RunHint>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunHint {
    pub id: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

/// 工作区体检报告。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub ok: bool,
    pub items: Vec<HealthItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HealthItem {
    pub id: String,
    /// ok | warn | error
    pub level: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
}

/// 工作区 Git 状态摘要（供标题栏按钮展示）。
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub dirty: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ahead: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behind: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ActionPrefs {
    #[serde(default)]
    pub recent_by_project_key: std::collections::HashMap<String, Vec<String>>,
    #[serde(default)]
    pub pinned_by_project_key: std::collections::HashMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OutdatedDependency {
    pub name: String,
    pub current: String,
    pub wanted: String,
    pub latest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}
