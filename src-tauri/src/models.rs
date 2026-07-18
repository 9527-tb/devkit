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
