/**
 * 工作台编排：扫描、进程、日志、标签与面板状态。
 * 对应 DESIGN.md §12.2 — 体量门禁：逻辑不堆在 .vue
 */
import { computed, ref, watch, onMounted, onBeforeUnmount } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Empty, message } from "antdv-next";
import { previewMode, settings, settingsPage, settingsCat, SETTINGS_CAT, normalizeSettingsCat } from "../../stores/settings.js";
import { toolsPage } from "../../stores/tools.js";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  root,
  workspaceHistory,
  workspaceOpenTick,
  setWorkspaceRoot,
  persistWorkspaceRoot,
  clearWorkspaceHistory,
  historyLabel,
  setLastScannedProjects,
  workspaceConfig,
} from "../../stores/workspace.js";
import { chooseNewWorkspace } from "./useWorkspacePicker.js";
import { normalizeProjectFilter } from "../settings/projectFilter.js";
import { resolveHealthNavigation } from "./useWorkspaceHealth.js";
import {
  healthReport,
  healthLoading,
  gitStatus,
  refreshWorkspaceHealth,
  refreshWorkspaceGit,
  projectHealthItems,
} from "../../stores/workspaceStatus.js";
import { makeProjectKey, normalizeProjectPath } from "../../shared/projectKey.js";
import { projectPassesFilter } from "../settings/projectFilter.js";
import {
  openTabs,
  activeKey as activeTabKey,
  panelByKey,
  openTab,
  activateTab,
  closeTab,
  closeTabs,
  setPanel,
} from "../../stores/projectTabs.js";
import { ingestProcesses, procsFor } from "../../stores/instances.js";

export function useWorkbench(tFn) {
const emptySimpleImage = Empty.PRESENTED_IMAGE_SIMPLE;
const t = tFn;

function depGroup(scope, children) {
  return {
    key: scope,
    name: scope,
    version: t("depItems", { n: children.length }),
    scope: "group",
    children: children.map((item) => ({
      ...item,
      key: item.key || `${scope}:${item.name}`,
      scope,
    })),
  };
}

function countDeps(nodes = []) {
  return nodes.reduce((sum, node) => {
    if (node.children?.length) return sum + countDeps(node.children);
    return sum + 1;
  }, 0);
}

function toDependencyTree(deps = []) {
  if (!deps.length) return [];
  // 已有分组树：清掉后端遗留的写死中文计数，交由 DepsPanel i18n 显示
  if (deps.some((d) => d.children?.length || d.scope === "group")) {
    return stripGroupVersionLabels(deps);
  }
  const groups = {};
  for (const dep of deps) {
    const scope = dep.scope || "dependencies";
    (groups[scope] ||= []).push({
      ...dep,
      key: dep.key || `${scope}:${dep.name}`,
    });
  }
  return Object.entries(groups).map(([scope, children]) => depGroup(scope, children));
}

function stripGroupVersionLabels(nodes = []) {
  return nodes.map((node) => {
    const next = { ...node };
    if (next.scope === "group") next.version = "";
    if (Array.isArray(next.children)) {
      next.children = stripGroupVersionLabels(next.children);
    }
    return next;
  });
}

const projects = ref([]);
const selectedPath = ref("");
const selectedKind = ref("");
const selectedPid = ref(null);
const activeTab = ref("logs");
const loading = ref(false);
const depLoading = ref(false);
const processes = ref({});
const logs = ref([]);
const liveDependencies = ref(null);
/** projectKey -> 最近一次 RunSummary */
const lastRunSummaries = ref({});
/** 最近/置顶动作偏好 */
const actionPrefs = ref({ recentByProjectKey: {}, pinnedByProjectKey: {} });
const currentRunSummary = computed(() => {
  const path = selectedPath.value;
  const kind = selectedKind.value;
  if (!path || !kind) return null;
  const key = makeProjectKey(normPath(path), kind);
  return lastRunSummaries.value[key] || null;
});

async function notifyTaskDone(summary) {
  if (settings.value.general?.notifyOnTaskDone === false) return;
  try {
    const focused = await getCurrentWindow().isFocused();
    if (focused) return;
  } catch {
    /* 继续尝试通知 */
  }
  try {
    const mod = await import("@tauri-apps/plugin-notification");
    let granted = await mod.isPermissionGranted();
    if (!granted) {
      granted = (await mod.requestPermission()) === "granted";
    }
    if (!granted) return;
    const proj =
      projects.value.find(
        (p) => makeProjectKey(normPath(p.path), p.kind) === summary.projectKey,
      )?.name || summary.path || t("projectFallback");
    const ok = summary.success;
    mod.sendNotification({
      title: ok ? t("notifyTaskOk") : t("notifyTaskFail"),
      body: `${proj} · ${summary.action || ""}`,
    });
  } catch {
    /* 插件未安装或权限拒绝 */
  }
}

function sameLogLines(a, b) {
  if (a === b) return true;
  if (!a || !b || a.length !== b.length) return false;
  for (let i = a.length - 1; i >= 0; i -= 1) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

/** 启动期高频日志：合并后每帧刷一次，避免主线程卡顿 */
let pendingLogLines = [];
let logFlushScheduled = false;
function flushPendingLogs() {
  logFlushScheduled = false;
  if (!pendingLogLines.length) return;
  const key = currentLogKey();
  if (!key) {
    pendingLogLines = [];
    return;
  }
  const chunk = pendingLogLines;
  pendingLogLines = [];
  if (!logsBoundKey) logsBoundKey = key;
  if (logsBoundKey !== key) return;
  const next = logs.value.length ? logs.value.concat(chunk) : chunk;
  logs.value = next.length > 3000 ? next.slice(-2200) : next;
}
function enqueueLogLine(line) {
  pendingLogLines.push(line);
  if (logFlushScheduled) return;
  logFlushScheduled = true;
  requestAnimationFrame(flushPendingLogs);
}

const GROUP_ORDER = ["Node", "Maven", "Gradle", "Cargo"];
const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

function visibleProjects() {
  const filter = settings.value.projectFilter;
  return projects.value.filter((project) => projectPassesFilter(project, filter));
}

const grouped = computed(() => {
  const map = visibleProjects().reduce((out, project) => {
    (out[project.group] ||= []).push(project);
    return out;
  }, {});
  const keys = [
    ...GROUP_ORDER.filter((g) => map[g]?.length),
    ...Object.keys(map)
      .filter((g) => !GROUP_ORDER.includes(g))
      .sort(),
  ];
  return keys.map((group) => ({ group, items: map[group] }));
});

const current = computed(() =>
  projects.value.find(
    (p) =>
      normPath(p.path) === normPath(selectedPath.value) && p.kind === selectedKind.value,
  ),
);
const currentProcesses = computed(() => projectProcs(selectedPath.value, selectedKind.value));
// 对齐原型：有运行实例即显示实例选择
const showInstanceSelect = computed(() => currentProcesses.value.length > 0);
const instanceOptions = computed(() =>
  currentProcesses.value.map((proc) => ({
    value: proc.pid,
    label: t("pidLabel", { pid: proc.pid }),
  })),
);
const dependencyTree = computed(() =>
  toDependencyTree(liveDependencies.value || current.value?.dependencies || []),
);
const dependencyCount = computed(() => countDeps(dependencyTree.value));

const refreshHealth = refreshWorkspaceHealth;

function orderedActionsFor(project) {
  if (!project) return [];
  const all = projectActionKeys(project);
  const key = projectKey(project);
  const pinned = (actionPrefs.value.pinnedByProjectKey[key] || []).filter((a) =>
    all.includes(a),
  );
  const recent = (actionPrefs.value.recentByProjectKey[key] || []).filter(
    (a) => all.includes(a) && !pinned.includes(a),
  );
  const rest = all.filter((a) => !pinned.includes(a) && !recent.includes(a));
  return [...pinned, ...recent, ...rest];
}

const actionList = computed(() => orderedActionsFor(current.value));

async function loadActionPrefs() {
  if (previewMode.value) return;
  try {
    actionPrefs.value = await invoke("load_action_prefs");
  } catch (error) {
    console.error(error);
  }
}

async function saveActionPrefs() {
  if (previewMode.value) return;
  try {
    await invoke("save_action_prefs", { prefs: actionPrefs.value });
  } catch (error) {
    console.error(error);
  }
}

function recordRecentAction(project, action) {
  const key = projectKey(project);
  if (!key || !action) return;
  const prev = actionPrefs.value.recentByProjectKey[key] || [];
  const next = [action, ...prev.filter((a) => a !== action)].slice(0, 8);
  actionPrefs.value = {
    ...actionPrefs.value,
    recentByProjectKey: {
      ...actionPrefs.value.recentByProjectKey,
      [key]: next,
    },
  };
  saveActionPrefs();
}

function pinAction(project, action) {
  const key = projectKey(project);
  if (!key || !action) return;
  const prev = actionPrefs.value.pinnedByProjectKey[key] || [];
  if (prev.includes(action)) return;
  actionPrefs.value = {
    ...actionPrefs.value,
    pinnedByProjectKey: {
      ...actionPrefs.value.pinnedByProjectKey,
      [key]: [...prev, action],
    },
  };
  saveActionPrefs();
}

function unpinAction(project, action) {
  const key = projectKey(project);
  if (!key || !action) return;
  const prev = actionPrefs.value.pinnedByProjectKey[key] || [];
  actionPrefs.value = {
    ...actionPrefs.value,
    pinnedByProjectKey: {
      ...actionPrefs.value.pinnedByProjectKey,
      [key]: prev.filter((a) => a !== action),
    },
  };
  saveActionPrefs();
}

function isPinnedAction(project, action) {
  const key = projectKey(project);
  return (actionPrefs.value.pinnedByProjectKey[key] || []).includes(action);
}

const refreshGitStatus = refreshWorkspaceGit;

async function applyWorkspaceConfigFromRoot() {
  workspaceConfig.value = null;
  if (!root.value || previewMode.value) return;
  try {
    const loaded = await invoke("load_workspace_config", { root: root.value });
    workspaceConfig.value = loaded || null;
    if (loaded?.projectFilter && settings.value.general?.preferWorkspaceConfig !== false) {
      settings.value = {
        ...settings.value,
        projectFilter: normalizeProjectFilter(loaded.projectFilter),
      };
    }
  } catch (error) {
    console.error(error);
  }
}

async function exportWorkspaceConfig() {
  if (!root.value || previewMode.value) {
    message.info(t("previewNeedTauri"));
    return;
  }
  try {
    const config = {
      version: 1,
      projectFilter: settings.value.projectFilter,
      pipelines: workspaceConfig.value?.pipelines || [],
      probes: workspaceConfig.value?.probes || {},
    };
    await invoke("save_workspace_config", { root: root.value, config });
    workspaceConfig.value = config;
    message.success(t("workspaceConfigExported"));
  } catch (error) {
    message.error(String(error));
  }
}

async function openInEditor(project) {
  if (!project?.path || previewMode.value) {
    message.info(t("previewNeedTauri"));
    return;
  }
  try {
    await invoke("open_in_editor", { path: project.path });
  } catch (error) {
    message.error(String(error));
  }
}

async function openInTerminal(project) {
  if (!project?.path || previewMode.value) {
    message.info(t("previewNeedTauri"));
    return;
  }
  try {
    await invoke("open_in_terminal", { path: project.path });
  } catch (error) {
    message.error(String(error));
  }
}

function handleHealthNavigate({ action, port }, navigate) {
  const target = resolveHealthNavigation(action);
  if (!target) return;
  if (target.type === "settings") {
    navigate?.openSettings?.(target.cat);
  } else if (target.type === "tools") {
    navigate?.openTools?.(target.toolId || "ports", { port });
  }
}

function projectActionKeys(project) {
  if (!project) return [];
  if (isMavenProject(project)) return mavenActionsFor(project);
  if (project.kind === "cargo" || project.kind === "gradle") {
    return [...(project.scripts || [])];
  }
  return ["install", ...(project.scripts || []).map((name) => `script:${name}`)];
}

function projectMenuItems(project) {
  const items = projectActionKeys(project).map((action) => ({
    key: action,
    label: displayAction(action),
  }));
  if (projectProcs(project.path, project.kind).length) {
    items.push({ type: "divider" });
    items.push({ key: "__stop__", label: t("stop"), danger: true });
  }
  return items;
}

function ensureActiveVisible() {
  if (!selectedPath.value) return;
  const active = current.value;
  if (active && projectPassesFilter(active, settings.value.projectFilter)) return;
  const first = visibleProjects()[0];
  if (first) {
    selectProject(first);
    return;
  }
  selectedPath.value = "";
  selectedKind.value = "";
  selectedPid.value = null;
  logs.value = [];
}

async function onProjectMenuClick(project, info) {
  const key = String(info.key);
  if (key === "__stop__") {
    await stop(project);
    return;
  }
  await run(project, key);
}

function normPath(path) {
  return normalizeProjectPath(path);
}

function isLogPrefix(short, long) {
  if (!short?.length || !long?.length || short.length > long.length) return false;
  for (let i = 0; i < short.length; i += 1) {
    if (short[i] !== long[i]) return false;
  }
  return true;
}

function displayAction(action) {
  return String(action || "").replace("script:", "");
}

function projectKey(project) {
  return makeProjectKey(project.path, project.kind);
}

function isSelected(project) {
  return (
    normPath(selectedPath.value) === normPath(project.path) &&
    selectedKind.value === project.kind
  );
}

function projectProcs(path, kind) {
  const fromStore = procsFor(path, kind);
  if (fromStore.length) return fromStore;
  const list = processes.value[path] || [];
  if (!kind) return list;
  return list.filter((p) => !p.kind || p.kind === kind);
}

const closeConfirmOpen = ref(false);
const closeConfirmTab = ref(null);
/** @type {import('vue').Ref<'single'|'batch'>} */
const closeConfirmMode = ref("single");
/** 批量关闭待执行的标签（确认后一次性关闭） */
const closeConfirmBatch = ref([]);

function selectProject(project, pid) {
  openTab(project);
  selectedPath.value = project.path;
  selectedKind.value = project.kind;
  liveDependencies.value = null;
  const procs = projectProcs(project.path, project.kind);
  if (pid != null) {
    selectedPid.value = pid;
    return;
  }
  selectedPid.value = procs[0]?.pid ?? null;
}

function syncSelectedPid() {
  const procs = currentProcesses.value;
  if (!procs.length) {
    selectedPid.value = null;
    return;
  }
  const current = selectedPid.value == null ? null : Number(selectedPid.value);
  if (current == null || !procs.some((p) => Number(p.pid) === current)) {
    selectedPid.value = procs[0].pid;
  }
}

function isMavenProject(project) {
  return project.kind === "maven";
}

function mavenActionsFor(project) {
  const actions = ["clean", "install", "clean install", "package"];
  if (project.springBoot) actions.push("spring-boot:run");
  return actions;
}

function mavenActions(items) {
  const actions = [
    { key: "clean", label: "clean" },
    { key: "install", label: "install" },
    { key: "clean install", label: "clean install" },
    { key: "package", label: "package" },
  ];
  if (items.some((p) => p.springBoot)) {
    actions.push({ key: "spring-boot:run", label: "spring-boot:run" });
  }
  return actions;
}

function nodeActionsFor(items) {
  const scripts = new Set();
  for (const p of items) {
    if (p.kind === "node") p.scripts.forEach((s) => scripts.add(s));
  }
  const preferred = ["dev", "start", "build", "preview", "lint"];
  const ordered = [
    ...preferred.filter((s) => scripts.has(s)),
    ...[...scripts].filter((s) => !preferred.includes(s)).sort(),
  ];
  return [
    { key: "install", label: "install" },
    ...ordered.map((s) => ({ key: `script:${s}`, label: s })),
  ];
}

function cargoActionsFor(items) {
  const scripts = new Set();
  for (const p of items) {
    if (p.kind === "cargo") (p.scripts || []).forEach((s) => scripts.add(s));
  }
  const preferred = ["build", "check", "test", "run", "clean"];
  const ordered = [
    ...preferred.filter((s) => scripts.has(s)),
    ...[...scripts].filter((s) => !preferred.includes(s)).sort(),
  ];
  return ordered.map((s) => ({ key: s, label: s }));
}

function gradleActionsFor(items) {
  const scripts = new Set();
  for (const p of items) {
    if (p.kind === "gradle") (p.scripts || []).forEach((s) => scripts.add(s));
  }
  const preferred = ["bootRun", "build", "test", "clean"];
  const ordered = [
    ...preferred.filter((s) => scripts.has(s)),
    ...[...scripts].filter((s) => !preferred.includes(s)).sort(),
  ];
  return ordered.map((s) => ({ key: s, label: s }));
}

function groupMenuItems(items) {
  if (!items.length) return [];
  const kind = items[0].kind;
  const actions =
    kind === "maven"
      ? mavenActions(items)
      : kind === "cargo"
        ? cargoActionsFor(items)
        : kind === "gradle"
          ? gradleActionsFor(items)
          : nodeActionsFor(items);
  const hasRunning = items.some((p) => projectProcs(p.path, p.kind).length);
  if (!hasRunning) return actions;
  return [...actions, { type: "divider" }, { key: "__stop__", label: t("stopAll"), danger: true }];
}

async function onGroupMenuClick(items, info) {
  const key = String(info.key);
  if (key === "__stop__") {
    await runGroup(items, "__stop__");
    return;
  }
  await runGroup(items, key);
}

async function refreshProcesses() {
  if (previewMode.value) return;
  try {
    processes.value = await invoke("running_processes");
    ingestProcesses(processes.value, projects.value);
    syncSelectedPid();
  } catch (error) {
    console.error(error);
  }
}

function onActivateTab(key) {
  activateTab(key);
  const tab = openTabs.value.find((t) => t.key === key);
  if (!tab) return;
  selectedPath.value = tab.path;
  selectedKind.value = tab.kind;
  syncSelectedPid();
  // 日志刷新由 selectedPath/selectedKind watch 统一处理，避免重复请求竞态
}

function syncSelectionAfterClose() {
  if (activeTabKey.value) onActivateTab(activeTabKey.value);
  else {
    selectedPath.value = "";
    selectedKind.value = "";
  }
}

function requestCloseTab(tab) {
  if (projectProcs(tab.path, tab.kind).length) {
    closeConfirmMode.value = "single";
    closeConfirmTab.value = tab;
    closeConfirmBatch.value = [tab];
    closeConfirmOpen.value = true;
    return;
  }
  closeTab(tab.key);
  syncSelectionAfterClose();
}

function resolveContextCloseTargets(action, fromTab) {
  const tabs = openTabs.value;
  const idx = tabs.findIndex((t) => t.key === fromTab?.key);
  if (action === "all") return [...tabs];
  if (action === "others") {
    if (idx < 0 || !fromTab?.key) return [];
    return tabs.filter((t) => t.key !== fromTab.key);
  }
  if (action === "right") return idx >= 0 ? tabs.slice(idx + 1) : [];
  if (action === "idle") return tabs.filter((t) => !tabIsRunning(t));
  if (action === "running") return tabs.filter((t) => tabIsRunning(t));
  return [];
}

function onTabContextAction({ action, tab }) {
  const targets = resolveContextCloseTargets(action, tab);
  if (!targets.length) return;
  const running = targets.filter((t) => tabIsRunning(t));
  if (running.length) {
    closeConfirmMode.value = "batch";
    closeConfirmTab.value = null;
    closeConfirmBatch.value = targets;
    closeConfirmOpen.value = true;
    return;
  }
  closeTabs(targets.map((t) => t.key));
  syncSelectionAfterClose();
}

async function applyCloseTabs(targets) {
  for (const tab of targets) {
    if (tabIsRunning(tab)) {
      try {
        await stop(
          { path: tab.path, kind: tab.kind, name: tab.name },
          { quiet: true },
        );
      } catch {
        /* still close */
      }
    }
  }
  closeTabs(targets.map((t) => t.key));
  syncSelectionAfterClose();
}

async function confirmCloseTab() {
  const targets = closeConfirmBatch.value.length
    ? [...closeConfirmBatch.value]
    : closeConfirmTab.value
      ? [closeConfirmTab.value]
      : [];
  closeConfirmOpen.value = false;
  closeConfirmTab.value = null;
  closeConfirmBatch.value = [];
  closeConfirmMode.value = "single";
  if (!targets.length) return;
  await applyCloseTabs(targets);
}

function cancelCloseConfirm() {
  closeConfirmOpen.value = false;
  closeConfirmTab.value = null;
  closeConfirmBatch.value = [];
  closeConfirmMode.value = "single";
}

function tabIsRunning(tab) {
  return projectProcs(tab.path, tab.kind).length > 0;
}

const closeConfirmRunningCount = computed(
  () => closeConfirmBatch.value.filter((tab) => tabIsRunning(tab)).length,
);

const activePanel = computed(() => {
  const key =
    activeTabKey.value ||
    projectKey({ path: selectedPath.value, kind: selectedKind.value });
  return panelByKey.value[key] || activeTab.value || "logs";
});

function switchPanel(id) {
  activeTab.value = id;
  const key =
    activeTabKey.value ||
    projectKey({ path: selectedPath.value, kind: selectedKind.value });
  if (key && key !== "::") setPanel(key, id);
}

const runtimePill = computed(() => {
  const kind = current.value?.kind || selectedKind.value;
  if (kind === "maven") return "Java";
  if (kind === "node") return "Node";
  if (kind === "cargo") return "Cargo";
  if (kind === "gradle") return "Gradle";
  return kind || "";
});

let logsFetchSeq = 0;
/** 当前 logs 绑定的项目键；用于防止空快照/旧请求覆盖 */
let logsBoundKey = "";

function currentLogKey(path = selectedPath.value, kind = selectedKind.value) {
  if (!path || !kind) return "";
  return makeProjectKey(path, kind);
}

async function refreshLogs() {
  const path = selectedPath.value;
  const kind = selectedKind.value;
  const key = currentLogKey(path, kind);
  if (!key) {
    pendingLogLines = [];
    logsBoundKey = "";
    logs.value = [];
    return;
  }
  if (previewMode.value) return;
  const seq = ++logsFetchSeq;
  const reqPath = normPath(path);
  try {
    const remote = await invoke("project_logs", { path: reqPath, kind });
    if (seq !== logsFetchSeq || currentLogKey() !== key) {
      return;
    }
    const next = Array.isArray(remote) ? remote.slice() : [];
    // 拉取期间到达的实时行不要丢弃
    if (pendingLogLines.length) {
      next.push(...pendingLogLines);
      pendingLogLines = [];
    }
    // 命令结束竞态：远端短暂空/落后时，绝不覆盖本地已追加日志
    if (logsBoundKey === key && logs.value.length) {
      if (!next.length) return;
      if (next.length < logs.value.length && isLogPrefix(next, logs.value)) return;
    }
    logsBoundKey = key;
    if (sameLogLines(logs.value, next)) return;
    logs.value = next;
  } catch (error) {
    if (seq === logsFetchSeq) message.error(String(error));
  }
}

async function openWorkspace(path) {
  if (!path) return;
  setWorkspaceRoot(path);
}

async function chooseDirectory() {
  await chooseNewWorkspace(t);
}

function dropdownPopupContainer() {
  return document.body;
}

const workspaceHistoryMenu = computed(() => {
  const items = workspaceHistory.value.map((path) => ({
    key: path,
    label: historyLabel(path),
    title: path,
  }));
  if (!items.length) {
    return {
      items: [{ key: "__empty", label: t("workspaceHistoryEmpty"), disabled: true }],
    };
  }
  return {
    items: [
      {
        type: "group",
        label: t("workspaceHistory"),
        children: items,
      },
      { type: "divider" },
      {
        key: "__clear",
        label: t("clearWorkspaceHistory"),
        danger: true,
      },
    ],
    onClick: ({ key }) => {
      if (key === "__clear") {
        clearWorkspaceHistory();
        return;
      }
      if (key === "__empty") return;
      openWorkspace(String(key));
    },
  };
});

async function scan() {
  if (!root.value) return message.warning(t("needWorkspaceDir"));
  if (previewMode.value) {
    message.info(t("previewNeedTauri"));
    return;
  }
  loading.value = true;
  try {
    persistWorkspaceRoot(root.value);
    projects.value = await invoke("scan_projects", { root: root.value });
    setLastScannedProjects(projects.value, root.value);
    liveDependencies.value = null;
    const first = visibleProjects()[0];
    if (first) selectProject(first);
    else {
      selectedPath.value = "";
      selectedKind.value = "";
      selectedPid.value = null;
      logs.value = [];
    }
    await refreshProcesses();
    await refreshLogs();
    await applyWorkspaceConfigFromRoot();
    await refreshHealth();
    await refreshGitStatus();
    message.success(t("scanFoundProjects", { n: projects.value.length }));
  } catch (error) {
    message.error(String(error));
  } finally {
    loading.value = false;
  }
}

async function run(project, action, { quiet = false } = {}) {
  activeTab.value = "logs";
  selectProject(project);
  if (previewMode.value) {
    message.info(t("previewNeedTauri"));
    return { ok: false, error: "preview" };
  }
  try {
    const view = await invoke("run_action", {
      path: normPath(project.path),
      action,
      kind: project.kind,
    });
    recordRecentAction(project, action);
    // 尽早绑定新实例 pid；日志靠事件流追加，此处不再 refreshLogs 以免空快照清空
    if (view?.pid != null) {
      selectedPid.value = view.pid;
    }
    await refreshProcesses();
    selectProject(project, view?.pid);
    if (!quiet) {
      message.success(
        t("actionExecuted", { name: project.name, action: displayAction(action) }),
      );
    }
    return { ok: true };
  } catch (error) {
    const text = String(error);
    message.error(`${project.name}：${text}`);
    if (text.includes("设置") || text.includes("JDK") || text.includes("Node") || text.includes("Settings")) {
      toolsPage.value = false;
      settingsPage.value = true;
      settingsCat.value = normalizeSettingsCat(SETTINGS_CAT.GENERAL);
    }
    return { ok: false, error: text };
  }
}

async function stop(project, { quiet = false } = {}) {
  if (previewMode.value) {
    message.info(t("previewNeedTauri"));
    return { ok: false, error: "preview" };
  }
  try {
    await invoke("stop_project", {
      path: normPath(project.path),
      kind: project.kind,
    });
    await refreshProcesses();
    syncSelectedPid();
    if (!quiet) message.success(t("projectStopped", { name: project.name }));
    return { ok: true };
  } catch (error) {
    message.error(String(error));
    return { ok: false, error: String(error) };
  }
}

async function stopInstance(project, pid) {
  if (previewMode.value) {
    message.info(t("previewNeedTauri"));
    return { ok: false, error: "preview" };
  }
  try {
    await invoke("stop_instance", { path: normPath(project.path), pid });
    await refreshProcesses();
    syncSelectedPid();
    message.success(t("instanceStopped", { name: project.name, pid }));
    return { ok: true };
  } catch (error) {
    message.error(String(error));
    return { ok: false, error: String(error) };
  }
}

async function runGroup(items, action) {
  const targets = items.filter((p) => {
    if (action === "__stop__") return true;
    if (isMavenProject(p)) return mavenActionsFor(p).includes(action);
    if (p.kind === "cargo" || p.kind === "gradle") {
      return (p.scripts || []).includes(action);
    }
    if (action === "install") return true;
    return p.scripts.includes(action.replace("script:", ""));
  });
  if (!targets.length) {
    message.warning(t("groupNoTargets"));
    return;
  }
  let ok = 0;
  let fail = 0;
  for (const project of targets) {
    const result =
      action === "__stop__"
        ? await stop(project, { quiet: true })
        : await run(project, action, { quiet: true });
    if (result?.ok) ok += 1;
    else fail += 1;
  }
  const label = action === "__stop__" ? t("stopAll") : displayAction(action);
  if (fail === 0) message.success(t("batchDone", { action: label, ok }));
  else message.warning(t("batchPartial", { action: label, ok, fail }));
}

async function clearLogs() {
  const path = selectedPath.value;
  const kind = selectedKind.value;
  pendingLogLines = [];
  if (!path || !kind) {
    logsBoundKey = "";
    logs.value = [];
    return;
  }
  if (previewMode.value) {
    logsBoundKey = "";
    logs.value = [];
    return;
  }
  try {
    await invoke("clear_logs", { path: normPath(path), kind });
    logsBoundKey = currentLogKey(path, kind);
    logs.value = [];
  } catch (error) {
    message.error(String(error));
  }
}

async function refreshDependencies() {
  if (!current.value) return;
  if (previewMode.value) {
    message.info(t("previewNeedTauri"));
    return;
  }
  depLoading.value = true;
  try {
    liveDependencies.value = await invoke("refresh_dependencies", {
      path: current.value.path,
      kind: current.value.kind,
    });
    message.success(t("depsRefreshed"));
  } catch (error) {
    message.error(String(error));
  } finally {
    depLoading.value = false;
  }
}

watch([selectedPath, selectedKind], (curr, prev) => {
  liveDependencies.value = null;
  const [path, kind] = curr || [];
  const [prevPath, prevKind] = prev || [];
  if (normPath(path) === normPath(prevPath) && kind === prevKind) return;
  pendingLogLines = [];
  logsBoundKey = "";
  logs.value = [];
  refreshLogs();
});
// 实例启停/切换 pid 不清空日志：项目级追加
watch(currentProcesses, syncSelectedPid, { deep: true });
watch(
  () => settings.value.projectFilter,
  () => ensureActiveVisible(),
  { deep: true },
);

let refreshTimer;
let unlistenLog;
let unlistenRunFinished;
onMounted(async () => {
  if (previewMode.value) return;

  await loadActionPrefs();

  unlistenLog = await listen("project-log", (event) => {
    const payload = event.payload;
    if (!payload || typeof payload.line !== "string") return;
    if (normPath(payload.path) !== normPath(selectedPath.value)) return;
    if (payload.kind && payload.kind !== selectedKind.value) return;
    enqueueLogLine(payload.line);
  });

  unlistenRunFinished = await listen("run-finished", (event) => {
    const summary = event.payload;
    if (!summary?.projectKey) return;
    lastRunSummaries.value = {
      ...lastRunSummaries.value,
      [summary.projectKey]: summary,
    };
    notifyTaskDone(summary);
  });

  // 进程状态轮询；日志靠事件流，避免启动期频繁 invoke
  refreshTimer = window.setInterval(() => {
    refreshProcesses();
  }, 2000);

  if (root.value) {
    await scan();
  }
});

watch(workspaceOpenTick, async () => {
  if (!root.value || previewMode.value) return;
  await scan();
});

onBeforeUnmount(() => {
  if (refreshTimer) window.clearInterval(refreshTimer);
  if (typeof unlistenLog === "function") unlistenLog();
  if (typeof unlistenRunFinished === "function") unlistenRunFinished();
});


  return {
    root,
    projects,
    loading,
    previewMode,
    workspaceHistoryMenu,
    dropdownPopupContainer,
    chooseDirectory,
    scan,
    openTabs,
    activeTabKey,
    tabIsRunning,
    onActivateTab,
    requestCloseTab,
    grouped,
    selectedPath,
    selectedKind,
    projectProcs,
    groupMenuItems,
    onGroupMenuClick,
    projectMenuItems,
    onProjectMenuClick,
    selectProject,
    current,
    showInstanceSelect,
    selectedPid,
    instanceOptions,
    runtimePill,
    actionList,
    displayAction,
    currentProcesses,
    run,
    stop,
    stopInstance,
    activePanel,
    switchPanel,
    logs,
    currentRunSummary,
    lastRunSummaries,
    clearLogs,
    dependencyTree,
    dependencyCount,
    depLoading,
    refreshDependencies,
    emptySimpleImage,
    closeConfirmOpen,
    closeConfirmTab,
    closeConfirmMode,
    closeConfirmBatch,
    closeConfirmRunningCount,
    confirmCloseTab,
    cancelCloseConfirm,
    onTabContextAction,
    healthReport,
    healthLoading,
    refreshHealth,
    handleHealthNavigate,
    projectHealthItems,
    gitStatus,
    refreshGitStatus,
    actionPrefs,
    pinAction,
    unpinAction,
    isPinnedAction,
    openInEditor,
    openInTerminal,
    exportWorkspaceConfig,
    workspaceConfig,
    t: tFn,
  };
}
