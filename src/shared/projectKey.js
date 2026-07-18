/**
 * 项目身份键：统一前后端 `path::kind` 字符串规则。
 * 依赖：无；被 stores / api / features 共用。
 * 对应 DESIGN.md §12.2 / §16.1
 */

// TODO(rs-instance-key): 与 Rust ProjectRef 对齐并覆盖多实例场景 — DESIGN §16.1

/**
 * 由路径与 Kind 生成 projectKey。
 * @param {string} path 项目绝对路径
 * @param {string} kind 项目类型（如 node / maven）
 * @returns {string}
 */
export function normalizeProjectPath(path) {
  return String(path || "")
    .replace(/\\/g, "/")
    .replace(/\/+$/g, "");
}

export function makeProjectKey(path, kind) {
  return `${normalizeProjectPath(path)}::${kind || ""}`;
}

/**
 * 解析 projectKey 为 path / kind。
 * @param {string} key
 * @returns {{ path: string, kind: string }}
 */
export function parseProjectKey(key) {
  const raw = String(key || "");
  const idx = raw.lastIndexOf("::");
  if (idx < 0) return { path: raw, kind: "" };
  return { path: raw.slice(0, idx), kind: raw.slice(idx + 2) };
}
