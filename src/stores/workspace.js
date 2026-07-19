/**
 * 工作区状态：根目录、历史、项目列表快照。
 * 对应 DESIGN.md §12.2 stores/workspace
 */

import { ref } from "vue";

export const WORKSPACE_KEY = "devkit.workspace";
/** 历史列表（Popover 展示）；与 WORKSPACE_KEY（上次打开、重启恢复）独立存储 */
export const WORKSPACE_HISTORY_KEY = "devkit.workspace.history";
export const WORKSPACE_HISTORY_MAX = 12;

export const root = ref("");
export const projects = ref([]);
export const scanning = ref(false);
export const providersMeta = ref([]);

/** 最近一次扫描的全量项目（设置页偏好表用；侧栏过滤前的快照） */
export const lastScannedProjects = ref([]);
export const lastScannedRoot = ref("");

/** 当前工作区 `.devkit/workspace.json`（可分享覆盖层） */
export const workspaceConfig = ref(null);

/** 工作区被打开/切换时递增，工作台监听后触发 scan */
export const workspaceOpenTick = ref(0);

export function readStoredWorkspaceRoot() {
  try {
    return localStorage.getItem(WORKSPACE_KEY) || "";
  } catch {
    return "";
  }
}

function loadWorkspaceHistory() {
  try {
    const raw = localStorage.getItem(WORKSPACE_HISTORY_KEY);
    if (!raw) return [];
    const list = JSON.parse(raw);
    return Array.isArray(list)
      ? list.filter((p) => typeof p === "string" && p.trim()).slice(0, WORKSPACE_HISTORY_MAX)
      : [];
  } catch {
    return [];
  }
}

function persistWorkspaceHistory(list) {
  try {
    localStorage.setItem(WORKSPACE_HISTORY_KEY, JSON.stringify(list));
  } catch {
    /* ignore */
  }
}

export const workspaceHistory = ref(loadWorkspaceHistory());

/** 初始化：恢复上次工作区，并确保进入历史 */
const storedRoot = readStoredWorkspaceRoot();
if (storedRoot) {
  root.value = storedRoot;
  if (!workspaceHistory.value.includes(storedRoot)) {
    workspaceHistory.value = [storedRoot, ...workspaceHistory.value].slice(
      0,
      WORKSPACE_HISTORY_MAX,
    );
    persistWorkspaceHistory(workspaceHistory.value);
  }
}

export function workspaceBasename(path) {
  if (!path) return "";
  const parts = String(path).replace(/\\/g, "/").split("/").filter(Boolean);
  return parts[parts.length - 1] || path;
}

export function historyLabel(path) {
  if (!path) return "";
  const parts = path.replace(/\\/g, "/").split("/").filter(Boolean);
  if (parts.length <= 2) return path;
  return `…/${parts.slice(-2).join("/")}`;
}

export function setLastScannedProjects(list, workspaceRoot = "") {
  lastScannedProjects.value = Array.isArray(list) ? list : [];
  lastScannedRoot.value = workspaceRoot || "";
}

function persistStoredWorkspaceRoot(path) {
  try {
    if (path) {
      localStorage.setItem(WORKSPACE_KEY, path);
    } else {
      localStorage.removeItem(WORKSPACE_KEY);
    }
  } catch {
    /* ignore */
  }
}

export function clearWorkspaceHistory() {
  workspaceHistory.value = [];
  persistWorkspaceHistory([]);
  // 上次打开路径存于 WORKSPACE_KEY；仅清 history 无法阻止重启 auto-open
  persistStoredWorkspaceRoot("");
}

/** 写入 root + 历史，不触发扫描 */
export function persistWorkspaceRoot(path) {
  const value = String(path || "").trim();
  if (!value) return;
  root.value = value;
  persistStoredWorkspaceRoot(value);
  const next = [value, ...workspaceHistory.value.filter((p) => p !== value)].slice(
    0,
    WORKSPACE_HISTORY_MAX,
  );
  workspaceHistory.value = next;
  persistWorkspaceHistory(next);
}

/** 设置工作区并通知工作台扫描 */
export function setWorkspaceRoot(path) {
  persistWorkspaceRoot(path);
  workspaceOpenTick.value += 1;
}
