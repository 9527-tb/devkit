/**
 * RunPlan 图模型（无旧 rows 兼容）：Start / Step / End + edges。
 * 父节点出度 >1 = 并行；出度 =1 = 串行。DAG join 后继续。
 */

import { invoke } from "@tauri-apps/api/core";
import { makeProjectKey, normalizeProjectPath } from "../../shared/projectKey.js";

export const NODE_START = "start";
export const NODE_END = "end";
export const NODE_STEP = "step";

export function newPlanId() {
  return `plan_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 8)}`;
}

export function newNodeId(prefix = "n") {
  return `${prefix}_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 6)}`;
}

export function newEdgeId() {
  return `e_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 6)}`;
}

/**
 * @param {object} project
 * @param {string} rootPath
 */
export function projectRelPath(project, rootPath) {
  const abs = normalizeProjectPath(project.path);
  const root = normalizeProjectPath(rootPath);
  if (root && abs.startsWith(root + "/")) {
    return abs.slice(root.length + 1);
  }
  if (root && abs === root) return ".";
  return abs;
}

export function createEmptyPlanGraph() {
  return {
    nodes: [
      { id: NODE_START, type: NODE_START, position: { x: 80, y: 160 } },
      { id: NODE_END, type: NODE_END, position: { x: 520, y: 160 } },
    ],
    edges: [],
  };
}

function normalizePosition(pos) {
  if (!pos || typeof pos !== "object") return { x: 0, y: 0 };
  return {
    x: Number.isFinite(Number(pos.x)) ? Number(pos.x) : 0,
    y: Number.isFinite(Number(pos.y)) ? Number(pos.y) : 0,
  };
}

function normalizeNode(raw) {
  if (!raw || typeof raw !== "object") return null;
  const id = String(raw.id || "").trim();
  const type = String(raw.type || "").trim();
  if (!id) return null;
  const position = normalizePosition(raw.position);
  if (type === NODE_START || type === NODE_END) {
    return { id, type, position };
  }
  if (type !== NODE_STEP) return null;
  const kind = String(raw.kind || "").trim();
  const relPath = String(raw.relPath || "").trim();
  const action = String(raw.action || "").trim();
  if (!kind || !relPath || !action) return null;
  return {
    id,
    type: NODE_STEP,
    position,
    relPath,
    kind,
    rootId: raw.rootId ? String(raw.rootId) : "",
    action,
    wait: raw.wait === "complete" ? "complete" : "spawn",
  };
}

function normalizeEdge(raw, nodeIds) {
  if (!raw || typeof raw !== "object") return null;
  const source = String(raw.source || "").trim();
  const target = String(raw.target || "").trim();
  if (!source || !target || source === target) return null;
  if (!nodeIds.has(source) || !nodeIds.has(target)) return null;
  const id = String(raw.id || `${source}->${target}`).trim();
  return { id, source, target };
}

/** 有向图是否有环 */
export function graphHasCycle(nodes, edges) {
  const outs = new Map(nodes.map((n) => [n.id, []]));
  for (const e of edges) {
    outs.get(e.source)?.push(e.target);
  }
  const state = new Map(); // 0=unseen 1=stack 2=done
  function dfs(id) {
    const s = state.get(id) || 0;
    if (s === 1) return true;
    if (s === 2) return false;
    state.set(id, 1);
    for (const t of outs.get(id) || []) {
      if (dfs(t)) return true;
    }
    state.set(id, 2);
    return false;
  }
  for (const n of nodes) {
    if (dfs(n.id)) return true;
  }
  return false;
}

/**
 * @returns {{ ok: true } | { ok: false, reason: string }}
 */
export function validatePlanGraph(nodes, edges) {
  const list = Array.isArray(nodes) ? nodes : [];
  const eds = Array.isArray(edges) ? edges : [];
  const starts = list.filter((n) => n.type === NODE_START);
  const ends = list.filter((n) => n.type === NODE_END);
  if (starts.length !== 1) return { ok: false, reason: "needStart" };
  if (ends.length !== 1) return { ok: false, reason: "needEnd" };
  const steps = list.filter((n) => n.type === NODE_STEP);
  if (!steps.length) return { ok: false, reason: "needSteps" };
  if (!steps.every((n) => n.action && n.relPath && n.kind)) {
    return { ok: false, reason: "incompleteStep" };
  }
  if (graphHasCycle(list, eds)) return { ok: false, reason: "cycle" };

  const outs = new Map(list.map((n) => [n.id, []]));
  const ins = new Map(list.map((n) => [n.id, []]));
  for (const e of eds) {
    outs.get(e.source)?.push(e.target);
    ins.get(e.target)?.push(e.source);
  }
  if (!(outs.get(NODE_START) || outs.get(starts[0].id))?.length) {
    return { ok: false, reason: "startDisconnected" };
  }
  const endId = ends[0].id;
  if (!(ins.get(endId) || []).length) return { ok: false, reason: "endDisconnected" };

  // Start 可达 End
  const seen = new Set();
  const q = [starts[0].id];
  while (q.length) {
    const id = q.shift();
    if (seen.has(id)) continue;
    seen.add(id);
    for (const t of outs.get(id) || []) q.push(t);
  }
  if (!seen.has(endId)) return { ok: false, reason: "unreachableEnd" };

  return { ok: true };
}

/**
 * 仅接受图格式；旧 rows 计划返回 null（作废）。
 */
export function normalizeRunPlan(raw, source = "user") {
  if (!raw || typeof raw !== "object") return null;
  // 旧表格模型：有 rows 且无合法 nodes → 作废
  if (Array.isArray(raw.rows) && !Array.isArray(raw.nodes)) return null;

  const nodes = (Array.isArray(raw.nodes) ? raw.nodes : [])
    .map(normalizeNode)
    .filter(Boolean);
  const start = nodes.find((n) => n.type === NODE_START);
  const end = nodes.find((n) => n.type === NODE_END);
  if (!start || !end) return null;
  const oldStart = start.id;
  const oldEnd = end.id;
  start.id = NODE_START;
  end.id = NODE_END;

  const nodeIds = new Set(nodes.map((n) => n.id));
  const edges = (Array.isArray(raw.edges) ? raw.edges : [])
    .map((e) => {
      const source =
        e.source === oldStart ? NODE_START : e.source === oldEnd ? NODE_END : e.source;
      const target =
        e.target === oldStart ? NODE_START : e.target === oldEnd ? NODE_END : e.target;
      return normalizeEdge({ ...e, source, target }, nodeIds);
    })
    .filter(Boolean);

  // 去重边
  const edgeKey = new Set();
  const uniqueEdges = [];
  for (const e of edges) {
    const k = `${e.source}->${e.target}`;
    if (edgeKey.has(k)) continue;
    edgeKey.add(k);
    uniqueEdges.push(e);
  }

  if (!nodes.some((n) => n.type === NODE_STEP)) return null;

  const id = String(raw.id || newPlanId());
  const name = String(raw.name || "Untitled").trim() || "Untitled";
  const stopOnError = raw.stopOnError !== false;
  const rootPath = raw.rootPath ? normalizeProjectPath(raw.rootPath) : "";

  return {
    id,
    name,
    source: source === "workspace" ? "workspace" : "user",
    rootPath,
    stopOnError,
    nodes,
    edges: uniqueEdges,
  };
}

export function listMergedRunPlans(userPlans, configsByRoot = {}) {
  /** @type {Map<string, object>} */
  const map = new Map();
  for (const raw of Array.isArray(userPlans) ? userPlans : []) {
    const plan = normalizeRunPlan(raw, "user");
    if (plan) map.set(plan.id, plan);
  }
  for (const [rootPath, cfg] of Object.entries(configsByRoot || {})) {
    const list = cfg?.runPlans || [];
    if (!Array.isArray(list)) continue;
    for (const raw of list) {
      const plan = normalizeRunPlan(
        { ...raw, rootPath: raw.rootPath || rootPath },
        "workspace",
      );
      if (plan) map.set(plan.id, plan);
    }
  }
  return [...map.values()].sort((a, b) => a.name.localeCompare(b.name));
}

/**
 * 解析图中 step 节点对应的项目。
 * @returns {{
 *   steps: Map<string, { node: object, project: object }>,
 *   outs: Map<string, string[]>,
 *   ins: Map<string, string[]>,
 * }}
 */
export function resolvePlanGraph(plan, projects, roots) {
  const rootById = new Map((roots || []).map((r) => [r.id, r]));
  const rootByPath = new Map(
    (roots || []).map((r) => [normalizeProjectPath(r.path), r]),
  );
  const steps = new Map();
  for (const node of plan.nodes || []) {
    if (node.type !== NODE_STEP) continue;
    let root =
      (node.rootId && rootById.get(node.rootId)) ||
      (plan.rootPath && rootByPath.get(normalizeProjectPath(plan.rootPath))) ||
      null;
    if (!root && roots?.length === 1) root = roots[0];
    const abs =
      node.relPath === "."
        ? root?.path
        : root
          ? `${normalizeProjectPath(root.path)}/${node.relPath}`.replace(/\/+/g, "/")
          : node.relPath;
    const project = (projects || []).find(
      (p) =>
        normalizeProjectPath(p.path) === normalizeProjectPath(abs) &&
        p.kind === node.kind,
    );
    if (!project) continue;
    steps.set(node.id, { node, project });
  }

  const outs = new Map();
  const ins = new Map();
  for (const n of plan.nodes || []) {
    outs.set(n.id, []);
    ins.set(n.id, []);
  }
  for (const e of plan.edges || []) {
    outs.get(e.source)?.push(e.target);
    ins.get(e.target)?.push(e.source);
  }
  return { steps, outs, ins };
}

export async function saveUserRunPlan(settingsRef, plan, persistFn) {
  const normalized = normalizeRunPlan({ ...plan, source: "user" }, "user");
  if (!normalized) throw new Error("invalid plan graph");
  const list = Array.isArray(settingsRef.value.runPlans)
    ? [...settingsRef.value.runPlans]
    : [];
  const idx = list.findIndex((p) => p.id === normalized.id);
  if (idx >= 0) list[idx] = normalized;
  else list.push(normalized);
  settingsRef.value = { ...settingsRef.value, runPlans: list };
  if (typeof persistFn === "function") await persistFn();
  return normalized;
}

export async function saveWorkspaceRunPlan(rootPath, plan, existingConfig) {
  const normalized = normalizeRunPlan(
    { ...plan, source: "workspace", rootPath },
    "workspace",
  );
  if (!normalized) throw new Error("invalid plan graph");
  const prev = existingConfig || {};
  const list = Array.isArray(prev.runPlans) ? [...prev.runPlans] : [];
  const idx = list.findIndex((p) => p.id === normalized.id);
  if (idx >= 0) list[idx] = normalized;
  else list.push(normalized);
  const config = {
    version: 1,
    projectFilter: prev.projectFilter,
    runPlans: list,
    probes: prev.probes || {},
  };
  await invoke("save_workspace_config", { root: rootPath, config });
  return { config, plan: normalized };
}

export async function deleteUserRunPlan(settingsRef, planId, persistFn) {
  const list = Array.isArray(settingsRef.value.runPlans)
    ? settingsRef.value.runPlans.filter((p) => p.id !== planId)
    : [];
  settingsRef.value = { ...settingsRef.value, runPlans: list };
  if (typeof persistFn === "function") await persistFn();
}

export async function deleteWorkspaceRunPlan(rootPath, planId, existingConfig) {
  const prev = existingConfig || {};
  const list = (Array.isArray(prev.runPlans) ? prev.runPlans : []).filter(
    (p) => p.id !== planId,
  );
  const config = {
    version: 1,
    projectFilter: prev.projectFilter,
    runPlans: list,
    probes: prev.probes || {},
  };
  await invoke("save_workspace_config", { root: rootPath, config });
  return config;
}

export { makeProjectKey };
