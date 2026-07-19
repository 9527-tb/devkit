/**
 * 前端工具注册表。
 * 新增工具：1) 在此追加元数据 2) 实现 features/tools/<id>/Panel.vue + api.js
 * 与后端 tools::registry::builtin_tools 的 id 对齐。
 */

import { defineAsyncComponent } from "vue";

/** @typedef {{ id: string, titleKey: string, order: number, component: import('vue').Component }} ToolDefinition */

/** @type {ToolDefinition[]} */
export const TOOL_REGISTRY = [
  {
    id: "ports",
    titleKey: "toolPortManager",
    order: 10,
    component: defineAsyncComponent(() => import("./ports/PortManagerPanel.vue")),
  },
  {
    id: "deploy",
    titleKey: "toolDeploy",
    order: 20,
    component: defineAsyncComponent(() => import("../deploy/DeployPanel.vue")),
  },
];

export function toolsSorted() {
  return [...TOOL_REGISTRY].sort((a, b) => a.order - b.order);
}

export function defaultToolId() {
  return toolsSorted()[0]?.id || "ports";
}

export function findTool(id) {
  return TOOL_REGISTRY.find((t) => t.id === id) || null;
}
