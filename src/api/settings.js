/**
 * 运行时设置与首次初始化 IPC 封装。
 * 依赖：api/tauri；被 stores/settings、stores/init 调用。
 * 对应 DESIGN.md §12.2 api/settings.js
 */

import { invokeSafe, listenSafe } from "./tauri.js";

// TODO(rs-init-progress): DONE — 订阅 init-progress 并暴露给 InitModal — DESIGN §6.1 / §12.2

export async function runtimeSettingsExists() {
  return invokeSafe("runtime_settings_exists");
}

export async function loadRuntimeSettings() {
  return invokeSafe("load_runtime_settings");
}

export async function saveRuntimeSettings(settings) {
  return invokeSafe("save_runtime_settings", { settings });
}

export async function initializeRuntimeSettings() {
  return invokeSafe("initialize_runtime_settings");
}

export async function validateRuntimePath(kind, path) {
  return invokeSafe("validate_runtime_path", { kind, path });
}

/** 探测系统已安装的 npm / pnpm / yarn */
export async function detectNodePackageManagers() {
  return invokeSafe("detect_node_package_managers");
}

/**
 * 订阅首次初始化进度事件。
 * @param {(event: { payload?: unknown }) => void} handler
 */
export async function listenInitProgress(handler) {
  return listenSafe("init-progress", handler);
}
