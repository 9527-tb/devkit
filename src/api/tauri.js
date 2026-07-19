/**
 * Tauri IPC 安全包装：桌面模式 invoke/listen，预览模式降级。
 * 业务 command 在 features 内直接 invoke；本模块提供 isTauri / invokeSafe / listenSafe。
 */

/** 是否在 Tauri 桌面壳内运行 */
export const isTauri =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/**
 * 安全调用 Tauri command；预览模式返回 null。
 * @param {string} cmd
 * @param {Record<string, unknown>} [args]
 */
export async function invokeSafe(cmd, args) {
  if (!isTauri) {
    console.warn(`[preview] invoke skipped: ${cmd}`, args);
    return null;
  }
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke(cmd, args);
}

/**
 * 安全订阅 Tauri 事件；预览模式返回空卸载函数。
 * @param {string} event
 * @param {(event: unknown) => void} handler
 * @returns {Promise<() => void>}
 */
export async function listenSafe(event, handler) {
  if (!isTauri) {
    console.warn(`[preview] listen skipped: ${event}`);
    return () => {};
  }
  const { listen } = await import("@tauri-apps/api/event");
  return listen(event, handler);
}
