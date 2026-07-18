//! 项目 Kind 标识（开放字符串，非封闭枚举）。
//!
//! DONE(rs-registry): 与 Provider KIND 常量对齐

/// 稳定 Kind 标识，如 `node`、`maven`、`cargo`、`gradle`。
pub type KindId = &'static str;

pub const KIND_NODE: KindId = "node";
pub const KIND_MAVEN: KindId = "maven";
pub const KIND_CARGO: KindId = "cargo";
pub const KIND_GRADLE: KindId = "gradle";

pub fn all_builtin() -> &'static [KindId] {
    &[KIND_NODE, KIND_MAVEN, KIND_CARGO, KIND_GRADLE]
}
