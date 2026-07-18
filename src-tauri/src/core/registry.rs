//! Provider 注册表。
//!
//! DONE(rs-registry): 内置 Node/Maven Detector 注册；Cargo 在 R6 启用 — DESIGN §6.7

use crate::core::kinds::{self, KIND_CARGO, KIND_GRADLE, KIND_MAVEN, KIND_NODE};
use crate::providers::cargo::CargoDetector;
use crate::providers::gradle::GradleDetector;
use crate::providers::maven::MavenDetector;
use crate::providers::node::NodeDetector;
use crate::providers::traits::ProjectDetector;
use std::sync::Arc;

/// 简易注册表：持有 Detector 列表与 kind 启用状态。
pub struct ProviderRegistry {
    detectors: Vec<Arc<dyn ProjectDetector>>,
    enabled: Vec<&'static str>,
}

impl ProviderRegistry {
    /// 内置 Provider（Node + Maven + Cargo + Gradle）。
    pub fn builtin() -> Self {
        let detectors: Vec<Arc<dyn ProjectDetector>> = vec![
            Arc::new(NodeDetector),
            Arc::new(MavenDetector),
            Arc::new(CargoDetector),
            Arc::new(GradleDetector),
        ];
        Self {
            detectors,
            enabled: vec![KIND_NODE, KIND_MAVEN, KIND_CARGO, KIND_GRADLE],
        }
    }

    pub fn new() -> Self {
        Self::builtin()
    }

    pub fn detectors(&self) -> &[Arc<dyn ProjectDetector>] {
        &self.detectors
    }

    pub fn is_enabled(&self, kind: &str) -> bool {
        self.enabled.iter().any(|k| *k == kind)
    }

    pub fn list_kinds(&self) -> Vec<String> {
        let mut keys: Vec<String> = kinds::all_builtin().iter().map(|s| (*s).to_string()).collect();
        // 确保启用中的 kind 都在列表中
        for k in &self.enabled {
            if self.is_enabled(k) && !keys.iter().any(|x| x == *k) {
                keys.push((*k).to_string());
            }
        }
        let _ = (KIND_CARGO, KIND_GRADLE);
        keys.sort();
        keys
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::builtin()
    }
}
