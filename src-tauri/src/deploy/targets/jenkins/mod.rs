//! Jenkins 部署目标（规划）。
//!
//! TODO(deploy-target-jenkins): 生成 Jenkinsfile + API 建 Job — DESIGN §9.6

pub fn target_kind() -> &'static str {
    "jenkins"
}
