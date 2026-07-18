//! 项目生态 Provider 插件根。
//!
//! 对应 DESIGN.md §6.6 / §12.3。新增 Kind = 新子目录 + register。
//!
//! DONE(rs-provider-node): Node Provider
//! DONE(rs-provider-maven): Maven Provider
//! DONE(rs-provider-cargo): Cargo Provider 验证扩展性 — DESIGN R6
//! DONE(rs-provider-gradle): Gradle Provider

pub mod traits;
pub mod node;
pub mod maven;
pub mod cargo;
pub mod gradle;

/// 注册内置 Provider（触发各子模块链接；扫描经 ProviderRegistry::builtin）。
pub fn register_builtin() {
    let _ = (
        node::KIND,
        maven::KIND,
        cargo::KIND,
        gradle::KIND,
        node::toolchain::toolchain_id(),
        maven::toolchain::toolchain_id(),
    );
}
