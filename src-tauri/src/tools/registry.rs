//! 工具元数据注册表：前端导航可对齐 `id` / `title_key`。

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolMeta {
    pub id: &'static str,
    pub title_key: &'static str,
    pub order: u32,
}

/// 内置工具列表。新增工具时在此追加一项，并实现对应子模块。
pub fn builtin_tools() -> Vec<ToolMeta> {
    vec![ToolMeta {
        id: "ports",
        title_key: "toolPortManager",
        order: 10,
    }]
}
