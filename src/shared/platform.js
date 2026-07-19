/**
 * 前端平台探测：快捷键文案等 UI 差异。
 * 依赖：navigator；被 workbench / 日志面板提示使用。
 * 对应 DESIGN.md §5 / §12.2 shared/platform
 */

// TODO(plat-linux): 补充 Linux / Windows 修饰键与平台能力提示 — DESIGN §5.1

const ua =
  typeof navigator !== "undefined"
    ? `${navigator.platform || ""} ${navigator.userAgent || ""}`
    : "";

/** 是否运行在 macOS（含 iOS 设备 UA 粗判） */
export const isMac = /Mac|iPhone|iPad|iPod/i.test(ua);

/** Windows */
export const isWindows = /Win/i.test(ua);

/** Linux（排除 Android） */
export const isLinux = /Linux/i.test(ua) && !/Android/i.test(ua);

/**
 * 是否使用 HTML 自定义窗口按钮。
 * macOS：原生红绿灯（Overlay titleBar）；Windows / Linux：decorations:false + 自定义控件。
 */
export const useCustomWindowControls = !isMac;

/** 修饰键展示标签：macOS 为 ⌘，其它为 Ctrl */
export const modKeyLabel = isMac ? "⌘" : "Ctrl";
