//! ProjectDetector / 命令解析 trait。
//!
//! 对应 DESIGN.md §6.5–6.6。扫描引擎只调用 trait，不写语言分支。
//!
//! DONE(rs-registry): Detector 插件接口已落地 — DESIGN §6.7

use crate::models::Project;
use std::path::Path;

/// 目录探测结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeResult {
    Miss,
    Hit,
}

/// 项目识别器（每种 Kind 一个实现）。
pub trait ProjectDetector: Send + Sync {
    #[allow(dead_code)]
    fn kind(&self) -> &'static str;
    fn probe(&self, dir: &Path) -> ProbeResult;
    fn enrich(&self, dir: &Path) -> Result<Project, String>;
}

/// 将动作解析为可执行 (program, args)。
#[allow(dead_code)]
pub trait ActionResolver: Send + Sync {
    fn kind(&self) -> &'static str;
    fn resolve_command(
        &self,
        project: &Project,
        action: &str,
    ) -> Result<(String, Vec<String>), String>;
}
