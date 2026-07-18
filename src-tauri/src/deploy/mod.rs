//! 部署 / 交付子系统（规划骨架，无真实远程副作用）。
//!
//! 对应 DESIGN.md §9。
//!
//! TODO(deploy-engine-skel): Deploy Engine 阶段编排 — DESIGN §9.5
//! TODO(deploy-target-ssh): SSH Target Adapter — DESIGN §9.6
//! TODO(deploy-target-jenkins): Jenkins Adapter — DESIGN §9.6

#![allow(dead_code)]

pub mod model;
pub mod engine;
pub mod secrets;
pub mod targets;

/// 模块自检占位，确保被 lib 链接。
pub fn init_deploy_subsystem() {
    let _ = model::DeployRunStatus::Pending;
}
