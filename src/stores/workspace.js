/**
 * 工作区状态：根目录、项目列表、扫描中标记、Provider 元数据。
 * 依赖：api/projects；被 features/workbench 消费。
 * 对应 DESIGN.md §12.2 stores/workspace
 */

import { ref } from "vue";

// TODO(fe-workbench-view): 从 App.vue 迁出 chooseRoot / scan / projectByKey — DESIGN §12.2

export const WORKSPACE_KEY = "devkit.workspace";

export const root = ref("");
export const projects = ref([]);
export const scanning = ref(false);
export const providersMeta = ref([]);

/** 最近一次扫描的全量项目（设置页偏好表用；侧栏过滤前的快照） */
export const lastScannedProjects = ref([]);
export const lastScannedRoot = ref("");

export function readStoredWorkspaceRoot() {
  try {
    return localStorage.getItem(WORKSPACE_KEY) || "";
  } catch {
    return "";
  }
}

export function setLastScannedProjects(list, workspaceRoot = "") {
  lastScannedProjects.value = Array.isArray(list) ? list : [];
  lastScannedRoot.value = workspaceRoot || "";
}
