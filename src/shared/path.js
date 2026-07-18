/**
 * 路径规范化辅助：前后端展示与比较前统一格式。
 * 依赖：无；被 workspace / topbar / 日志匹配等使用。
 * 对应 DESIGN.md §5 / §12.2 shared/path
 */

// TODO(plat-windows): 补齐盘符、反斜杠与长路径规范化 — DESIGN §5.1

/**
 * 规范化路径（当前仅去掉尾部斜杠）。
 * @param {string} path
 * @returns {string}
 */
export function normPath(path) {
  return String(path || "").replace(/\/+$/g, "");
}
