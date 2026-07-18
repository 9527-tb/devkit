/**
 * 进程运行 / 停止 / 日志相关 IPC 封装。
 * 依赖：api/tauri；被 stores/instances 调用。
 * 对应 DESIGN.md §12.2 api/processes.js
 */

import { invokeSafe } from "./tauri.js";

// TODO(rs-instance-key): run/stop/logs 全面改为 projectKey + instanceId — DESIGN §16.1

/** @param {{ path: string, kind: string, action: string }} args */
export async function runAction(args) {
  return invokeSafe("run_action", args);
}

/** @param {{ path: string, kind?: string }} args */
export async function stopProject(args) {
  return invokeSafe("stop_project", args);
}

/** @param {{ path: string, pid: number }} args */
export async function stopInstance(args) {
  return invokeSafe("stop_instance", args);
}

/** @param {{ path: string, kind: string }} args */
export async function projectLogs(args) {
  return invokeSafe("project_logs", args);
}

/** @param {{ path: string, kind: string }} args */
export async function clearLogs(args) {
  return invokeSafe("clear_logs", args);
}

/** @param {{ pid: number }} args */
export async function processMetrics(args) {
  return invokeSafe("process_metrics", args);
}
