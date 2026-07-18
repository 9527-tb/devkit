/**
 * 前端平台探测：快捷键文案等 UI 差异。
 * 依赖：navigator；被 workbench / 日志面板提示使用。
 * 对应 DESIGN.md §5 / §12.2 shared/platform
 */

// TODO(plat-linux): 补充 Linux / Windows 修饰键与平台能力提示 — DESIGN §5.1

/** 是否运行在 macOS（含 iOS 设备 UA 粗判） */
export const isMac =
  typeof navigator !== "undefined" &&
  /Mac|iPhone|iPad|iPod/i.test(navigator.platform || navigator.userAgent || "");

/** 修饰键展示标签：macOS 为 ⌘，其它为 Ctrl */
export const modKeyLabel = isMac ? "⌘" : "Ctrl";
