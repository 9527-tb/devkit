/**
 * 工作区状态：多根会话、历史、项目列表快照。
 * 对应 DESIGN.md §12.2 stores/workspace；多根见 plan Multi-root and RunPlans
 */

import { computed, ref } from "vue";

export const WORKSPACE_KEY = "devkit.workspace";
/** 多根会话（主存储） */
export const WORKSPACE_SESSION_KEY = "devkit.workspace.session";
/** 历史列表（Popover 展示）；与 WORKSPACE_KEY（上次打开、重启恢复）独立存储 */
export const WORKSPACE_HISTORY_KEY = "devkit.workspace.history";
export const WORKSPACE_HISTORY_MAX = 12;

/**
 * @typedef {{ id: string, path: string, label: string, enabled: boolean }} WorkspaceRoot
 * @typedef {{ roots: WorkspaceRoot[] }} WorkspaceSession
 */

export const root = ref("");
/** @type {import('vue').Ref<WorkspaceRoot[]>} */
export const workspaceRoots = ref([]);
export const projects = ref([]);
export const scanning = ref(false);
export const providersMeta = ref([]);

/** 最近一次扫描的全量项目（设置页偏好表用；侧栏过滤前的快照） */
export const lastScannedProjects = ref([]);
export const lastScannedRoot = ref("");

/** 各根目录的 workspace.json 合并结果（主根优先展示） */
export const workspaceConfig = ref(null);
/** rootPath → WorkspaceConfig */
export const workspaceConfigsByRoot = ref({});

/** 工作区被打开/切换时递增，工作台监听后触发 scan */
export const workspaceOpenTick = ref(0);

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

function makeRootId(path) {
  return `root:${String(path || "").replace(/\\/g, "/")}`;
}

function normalizePath(path) {
  return String(path || "").trim().replace(/\\/g, "/").replace(/\/+$/, "");
}

/** @returns {WorkspaceRoot} */
export function makeWorkspaceRoot(path) {
  const p = normalizePath(path);
  return {
    id: makeRootId(p),
    path: p,
    label: workspaceBasename(p),
    enabled: true,
  };
}

function syncPrimaryRoot() {
  const enabled = workspaceRoots.value.filter((r) => r.enabled);
  const primary = enabled[0] || workspaceRoots.value[0];
  root.value = primary?.path || "";
  persistStoredWorkspaceRoot(root.value);
}

function persistSession() {
  try {
    const session = { roots: workspaceRoots.value };
    localStorage.setItem(WORKSPACE_SESSION_KEY, JSON.stringify(session));
  } catch {
    /* ignore */
  }
  syncPrimaryRoot();
}

function loadSessionFromStorage() {
  try {
    const raw = localStorage.getItem(WORKSPACE_SESSION_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      if (Array.isArray(parsed?.roots) && parsed.roots.length) {
        return parsed.roots
          .filter((r) => r && typeof r.path === "string" && r.path.trim())
          .map((r) => ({
            id: r.id || makeRootId(r.path),
            path: normalizePath(r.path),
            label: r.label || workspaceBasename(r.path),
            enabled: r.enabled !== false,
          }));
      }
    }
  } catch {
    /* ignore */
  }
  // 兼容旧单根
  const legacy = readStoredWorkspaceRoot();
  if (legacy) return [makeWorkspaceRoot(legacy)];
  return [];
}

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

/** 启用中的根目录 */
export const enabledWorkspaceRoots = computed(() =>
  workspaceRoots.value.filter((r) => r.enabled && r.path),
);

// 启动：恢复会话
workspaceRoots.value = loadSessionFromStorage();
syncPrimaryRoot();
if (root.value && !workspaceHistory.value.includes(root.value)) {
  workspaceHistory.value = [root.value, ...workspaceHistory.value].slice(
    0,
    WORKSPACE_HISTORY_MAX,
  );
  persistWorkspaceHistory(workspaceHistory.value);
}

export function setLastScannedProjects(list, workspaceRoot = "") {
  lastScannedProjects.value = Array.isArray(list) ? list : [];
  lastScannedRoot.value = workspaceRoot || root.value || "";
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

/** 清空扫描快照与工作区配置缓存 */
export function clearScannedWorkspaceState() {
  lastScannedProjects.value = [];
  lastScannedRoot.value = "";
  workspaceConfig.value = null;
  workspaceConfigsByRoot.value = {};
  projects.value = [];
}

export function clearWorkspaceHistory() {
  workspaceHistory.value = [];
  persistWorkspaceHistory([]);
  persistStoredWorkspaceRoot("");
  workspaceRoots.value = [];
  persistSession();
  clearScannedWorkspaceState();
  try {
    localStorage.removeItem(WORKSPACE_SESSION_KEY);
  } catch {
    /* ignore */
  }
  // 通知工作台清空侧栏项目 / 标签
  workspaceOpenTick.value += 1;
}

export function pushHistory(path) {
  const value = normalizePath(path);
  if (!value) return;
  const next = [value, ...workspaceHistory.value.filter((p) => p !== value)].slice(
    0,
    WORKSPACE_HISTORY_MAX,
  );
  workspaceHistory.value = next;
  persistWorkspaceHistory(next);
}

/** 写入 root + 历史，不触发扫描（兼容单根 API） */
export function persistWorkspaceRoot(path) {
  const value = normalizePath(path);
  if (!value) return;
  const existing = workspaceRoots.value.find((r) => normalizePath(r.path) === value);
  if (existing) {
    workspaceRoots.value = workspaceRoots.value.map((r) =>
      r.id === existing.id ? { ...r, enabled: true } : r,
    );
  } else {
    workspaceRoots.value = [makeWorkspaceRoot(value)];
  }
  persistSession();
  pushHistory(value);
}

/**
 * 添加根目录（不替换现有）。
 * @returns {{ ok: boolean, reason?: string }}
 */
export function addWorkspaceRoot(path, maxRoots = 10) {
  const value = normalizePath(path);
  if (!value) return { ok: false, reason: "empty" };
  if (workspaceRoots.value.some((r) => normalizePath(r.path) === value)) {
    return { ok: false, reason: "exists" };
  }
  const limit = Math.max(1, Number(maxRoots) || 10);
  if (workspaceRoots.value.length >= limit) {
    return { ok: false, reason: "limit" };
  }
  workspaceRoots.value = [...workspaceRoots.value, makeWorkspaceRoot(value)];
  persistSession();
  pushHistory(value);
  workspaceOpenTick.value += 1;
  return { ok: true };
}

export function removeWorkspaceRoot(id) {
  workspaceRoots.value = workspaceRoots.value.filter((r) => r.id !== id);
  persistSession();
  workspaceOpenTick.value += 1;
}

export function setRootEnabled(id, enabled) {
  workspaceRoots.value = workspaceRoots.value.map((r) =>
    r.id === id ? { ...r, enabled: !!enabled } : r,
  );
  persistSession();
  workspaceOpenTick.value += 1;
}

export function reorderRoots(ids) {
  const map = new Map(workspaceRoots.value.map((r) => [r.id, r]));
  const next = ids.map((id) => map.get(id)).filter(Boolean);
  for (const r of workspaceRoots.value) {
    if (!ids.includes(r.id)) next.push(r);
  }
  workspaceRoots.value = next;
  persistSession();
}

/** 替换为单根会话并触发扫描 */
export function setWorkspaceRoot(path) {
  const value = normalizePath(path);
  if (!value) return;
  workspaceRoots.value = [makeWorkspaceRoot(value)];
  persistSession();
  pushHistory(value);
  workspaceOpenTick.value += 1;
}

/** 会话显示名 */
export function sessionDisplayName(t) {
  const enabled = enabledWorkspaceRoots.value;
  if (!enabled.length) return "";
  if (enabled.length === 1) return enabled[0].label || workspaceBasename(enabled[0].path);
  if (typeof t === "function") {
    return t("multiWorkspaceLabel", { n: enabled.length });
  }
  return `${enabled[0].label} +${enabled.length - 1}`;
}
