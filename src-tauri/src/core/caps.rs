//! 面板 / 能力描述。
//!
//! TODO(rs-capabilities): 实现 get_capabilities 合并 Project+Instance — DESIGN §8

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    pub id: String,
    pub title_key: String,
    pub order: i32,
    /// project | instance | both
    pub scope: String,
}

pub fn default_caps_for_kind(kind: &str) -> Vec<Capability> {
    let caps = vec![
        Capability {
            id: "panel.logs".into(),
            title_key: "logs".into(),
            order: 10,
            scope: "instance".into(),
        },
        Capability {
            id: "panel.deps".into(),
            title_key: "deps".into(),
            order: 20,
            scope: "project".into(),
        },
        Capability {
            id: "panel.monitor".into(),
            title_key: "monitor".into(),
            order: 30,
            scope: "instance".into(),
        },
    ];
    // TODO(deploy-panel-stub): maven/node/cargo 可附加 panel.deploy
    let _ = kind;
    caps
}

/// IPC 用：返回能力 id 列表。
pub fn capabilities_for_kind(kind: &str) -> Vec<String> {
    default_caps_for_kind(kind)
        .into_iter()
        .map(|c| c.id)
        .collect()
}
