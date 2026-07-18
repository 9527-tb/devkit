/**
 * 面板注册表：panelId → Vue 组件；由 capabilities 驱动 Tab。
 * 依赖：各 *Panel.vue；被 PanelHost 使用。
 * 对应 DESIGN.md §8 / §12.2 panels/registry
 */

import { defineAsyncComponent } from "vue";

// DONE(fe-panel-registry): 注册 logs/deps/monitor/deploy — DESIGN §8.1

const LogsPanel = defineAsyncComponent(() => import("./LogsPanel.vue"));
const DepsPanel = defineAsyncComponent(() => import("./DepsPanel.vue"));
const MonitorPanel = defineAsyncComponent(() => import("./MonitorPanel.vue"));
const DeployPanel = defineAsyncComponent(() => import("../../deploy/DeployPanel.vue"));

/** @type {Record<string, unknown>} */
export const panelRegistry = {
  "panel.logs": LogsPanel,
  logs: LogsPanel,
  "panel.deps": DepsPanel,
  dependencies: DepsPanel,
  deps: DepsPanel,
  "panel.monitor": MonitorPanel,
  monitor: MonitorPanel,
  "panel.deploy": DeployPanel,
  deploy: DeployPanel,
};

/**
 * 按 id 解析面板组件。
 * @param {string} panelId
 */
export function resolvePanel(panelId) {
  return panelRegistry[panelId] || null;
}

/** 默认面板顺序（无 capabilities 时）。 */
export const DEFAULT_PANELS = [
  { id: "logs", titleKey: "logs", cap: "panel.logs" },
  { id: "dependencies", titleKey: "deps", cap: "panel.deps" },
  { id: "monitor", titleKey: "monitor", cap: "panel.monitor" },
];
