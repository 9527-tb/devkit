//! 密钥引用（不落明文）。
//!
//! TODO(deploy-engine-skel): SecretStore 对接 OS 钥匙串 — DESIGN §9.7 / §16.10

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretRef {
    pub id: String,
}
