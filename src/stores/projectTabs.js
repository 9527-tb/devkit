/**
 * 多项目标签页状态：打开列表、激活键、每 tab 的 panelId。
 * 依赖：shared/projectKey；被 Workbench / ProjectTabs 消费。
 * 对应 DESIGN.md §12.2 stores/projectTabs
 */

import { ref } from "vue";
import { makeProjectKey } from "../shared/projectKey.js";

// DONE(ux-project-tabs): open/close/activate — DESIGN §10.4

export const openTabs = ref([]);
export const activeKey = ref("");
/** @type {import('vue').Ref<Record<string, string>>} */
export const panelByKey = ref({});

/**
 * @param {{ path: string, kind: string, name: string }} project
 */
export function openTab(project) {
  const key = makeProjectKey(project.path, project.kind);
  if (!openTabs.value.some((t) => t.key === key)) {
    openTabs.value = [
      ...openTabs.value,
      {
        key,
        path: project.path,
        kind: project.kind,
        name: project.name,
      },
    ];
  }
  activeKey.value = key;
  if (!panelByKey.value[key]) {
    panelByKey.value = { ...panelByKey.value, [key]: "logs" };
  }
  return key;
}

export function activateTab(key) {
  activeKey.value = key;
}

export function closeTab(key) {
  closeTabs([key]);
}

/** @param {string[]} keys */
export function closeTabs(keys) {
  if (!keys?.length) return;
  const drop = new Set(keys);
  openTabs.value = openTabs.value.filter((t) => !drop.has(t.key));
  const next = { ...panelByKey.value };
  for (const key of drop) delete next[key];
  panelByKey.value = next;
  if (drop.has(activeKey.value)) {
    activeKey.value = openTabs.value[openTabs.value.length - 1]?.key || "";
  }
}

export function setPanel(key, panelId) {
  panelByKey.value = { ...panelByKey.value, [key]: panelId };
}
