/**
 * 端口管理工具 IPC（仅本工具使用）。
 */

import { invokeSafe } from "../../../api/tauri.js";

/**
 * @param {string|number} query
 * @param {boolean} [fuzzy=false]
 */
export async function lookupPort(query, fuzzy = false) {
  return invokeSafe("tool_ports_lookup", {
    query: String(query ?? "").trim(),
    fuzzy: !!fuzzy,
  });
}

/** @param {number} pid */
export async function killPid(pid) {
  return invokeSafe("tool_ports_kill", { pid });
}
