/**
 * 运行实例状态：按 projectKey 索引进程视图。
 * 对应 DESIGN.md §16.1
 */

import { computed, ref } from "vue";
import { makeProjectKey } from "../shared/projectKey.js";

// DONE(rs-instance-key): 前端按 projectKey 索引；兼容旧 path 键 — DESIGN §16.1

/** @type {import('vue').Ref<Record<string, Array>>} */
export const processesByKey = ref({});

/**
 * 合并后端返回的 processes map（可能是 path 或 path::kind 键）。
 * @param {Record<string, Array>} map
 * @param {Array<{path:string,kind:string}>} projects
 */
export function ingestProcesses(map, projects = []) {
  const next = {};
  for (const [key, list] of Object.entries(map || {})) {
    next[key] = list;
  }
  // 双读：若仅有 path 键，按项目 kind 投影
  for (const p of projects) {
    const pk = makeProjectKey(p.path, p.kind);
    if (!next[pk] && next[p.path]) {
      const filtered = (next[p.path] || []).filter(
        (proc) => !proc.kind || proc.kind === p.kind,
      );
      if (filtered.length) next[pk] = filtered;
    }
  }
  processesByKey.value = next;
}

export function procsFor(path, kind) {
  const pk = makeProjectKey(path, kind);
  return (
    processesByKey.value[pk] ||
    (processesByKey.value[path] || []).filter((p) => !p.kind || p.kind === kind) ||
    []
  );
}

export const runningCount = computed(() =>
  Object.values(processesByKey.value).reduce((n, list) => n + (list?.length || 0), 0),
);
