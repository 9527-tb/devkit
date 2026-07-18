/**
 * 部署 / 交付子系统前端 API（规划骨架）。
 * 依赖：api/tauri；首期不执行真实远程副作用。
 * 对应 DESIGN.md §9 / §17 R8
 */

import { invokeSafe } from "./tauri.js";

// TODO(deploy-engine-skel): Deploy Engine IPC 骨架（export / list targets）— DESIGN §9 / R8

/**
 * 占位：列出可用部署目标（尚未实现）。
 */
export async function listDeployTargets() {
  return invokeSafe("list_deploy_targets");
}
