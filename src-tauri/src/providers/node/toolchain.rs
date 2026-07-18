//! Node 工具链提示（engines / .nvmrc 等由 runtime 解析）。
//!
//! 运行时选择仍在 runtime::resolve_node；此处保留扩展点。

/// Provider 侧 toolchain id。
pub fn toolchain_id() -> &'static str {
    "node"
}
