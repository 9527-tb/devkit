/**
 * 项目扫描相关 IPC 封装。
 * 依赖：api/tauri；被 stores/workspace 调用。
 * 对应 DESIGN.md §12.2 api/projects.js
 */

import { invokeSafe } from "./tauri.js";

// TODO(rs-scan-engine): 对接 scan_projects / list_providers / get_capabilities — DESIGN §12.3

/**
 * 扫描工作区根目录下的子项目。
 * @param {string} root
 */
export async function scanProjects(root) {
  return invokeSafe("scan_projects", { root });
}
