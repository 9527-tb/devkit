/**
 * 设置状态：settings 对象、保存中标记、外观应用。
 * 依赖：api/settings、themes；被 SettingsView / App 壳消费。
 * 对应 DESIGN.md §12.2 stores/settings
 */

import { ref } from "vue";
import { message } from "antdv-next";
import { THEME_OPTIONS, applyThemeToDocument } from "../themes/index.js";
import { createTranslator, LOCALE_OPTIONS } from "../i18n/index.js";
import { isTauri } from "../api/tauri.js";
import { saveRuntimeSettings } from "../api/settings.js";
import {
  emptyProjectFilter,
  normalizeProjectFilter,
} from "../features/settings/projectFilter.js";

// TODO(fe-settings-store): DONE — load / patch / applyAppearance — DESIGN §12.2

export const SETTINGS_KEY = "devkit.settings";
export const previewMode = ref(!isTauri);

/** 功能按钮展示数量：默认 5，范围 1–10 */
export function clampActionButtonCount(value) {
  const n = Number(value);
  if (!Number.isFinite(n)) return 5;
  return Math.min(10, Math.max(1, Math.round(n)));
}

/** Node 执行工具：npm | pnpm | yarn，默认 npm */
export const NODE_PACKAGE_MANAGERS = ["npm", "pnpm", "yarn"];

export function normalizeNodePackageManager(value) {
  const v = String(value || "")
    .trim()
    .toLowerCase();
  return NODE_PACKAGE_MANAGERS.includes(v) ? v : "npm";
}

export function emptySettings() {
  return {
    general: {
      logWrap: true,
      notifyOnTaskDone: true,
      healthCheckOnScan: true,
      preferWorkspaceConfig: true,
      editorCommand: "",
      terminalApp: "",
      theme: "teal",
      locale: "zh-CN",
      actionButtonCount: 5,
      launchAtLogin: false,
      /** 关闭按钮默认最小化到托盘 */
      closeToTray: true,
      /** 启动后静默检查更新 */
      autoCheckUpdate: true,
    },
    java: { jdks: [], mavenHome: "" },
    node: { nodes: [], packageManager: "npm" },
    /** 全局侧栏过滤（按 Kind + 动作能力），与工作区/选中项无关 */
    projectFilter: emptyProjectFilter(),
  };
}

export function normalizeSettings(raw) {
  const base = emptySettings();
  if (!raw || typeof raw !== "object") return base;
  const themeId = THEME_OPTIONS.some((x) => x.id === raw.general?.theme)
    ? raw.general.theme
    : "teal";
  const localeId = LOCALE_OPTIONS.some((x) => x.id === raw.general?.locale)
    ? raw.general.locale
    : "zh-CN";
  return {
    general: {
      logWrap: raw.general?.logWrap !== false,
      notifyOnTaskDone: raw.general?.notifyOnTaskDone !== false,
      healthCheckOnScan: raw.general?.healthCheckOnScan !== false,
      preferWorkspaceConfig: raw.general?.preferWorkspaceConfig !== false,
      editorCommand: String(raw.general?.editorCommand || "").trim(),
      terminalApp: String(raw.general?.terminalApp || "").trim(),
      theme: themeId,
      locale: localeId,
      actionButtonCount: clampActionButtonCount(raw.general?.actionButtonCount),
      launchAtLogin: !!raw.general?.launchAtLogin,
      closeToTray: raw.general?.closeToTray !== false,
      autoCheckUpdate: raw.general?.autoCheckUpdate !== false,
    },
    java: {
      jdks: Array.isArray(raw.java?.jdks)
        ? raw.java.jdks
        : Array.isArray(raw.jdks)
          ? raw.jdks
          : [],
      mavenHome: raw.java?.mavenHome || "",
    },
    node: {
      nodes: Array.isArray(raw.node?.nodes)
        ? raw.node.nodes
        : Array.isArray(raw.nodes)
          ? raw.nodes
          : [],
      packageManager: normalizeNodePackageManager(raw.node?.packageManager),
    },
    projectFilter: normalizeProjectFilter(raw.projectFilter),
  };
}

export function entryKey(entry) {
  return entry.id || `${entry.version}:${entry.path}`;
}

export const settings = ref(emptySettings());
export const saving = ref(false);
export const settingsPage = ref(false);
export const settingsLoading = ref(false);

/** 设置侧栏分类：工具链 × 项目类型（与持久化字段解耦） */
export const SETTINGS_CAT = {
  GENERAL: "general",
  TOOLCHAIN_JDK: "toolchain-jdk",
  TOOLCHAIN_NODE: "toolchain-node",
  PROVIDER_MAVEN: "provider-maven",
  PROVIDER_GRADLE: "provider-gradle",
  PROVIDER_NODE: "provider-node",
  PROVIDER_CARGO: "provider-cargo",
};

const SETTINGS_CAT_SET = new Set(Object.values(SETTINGS_CAT));

/** 旧导航 id → 新 cat（兼容内存中的旧值） */
const LEGACY_SETTINGS_CAT = {
  java: SETTINGS_CAT.PROVIDER_MAVEN,
  node: SETTINGS_CAT.TOOLCHAIN_NODE,
  cargo: SETTINGS_CAT.PROVIDER_CARGO,
  gradle: SETTINGS_CAT.PROVIDER_GRADLE,
};

/** provider kind → settingsCat */
export const PROVIDER_KIND_CAT = {
  maven: SETTINGS_CAT.PROVIDER_MAVEN,
  gradle: SETTINGS_CAT.PROVIDER_GRADLE,
  node: SETTINGS_CAT.PROVIDER_NODE,
  cargo: SETTINGS_CAT.PROVIDER_CARGO,
};

export function normalizeSettingsCat(id) {
  const raw = String(id || "").trim();
  if (SETTINGS_CAT_SET.has(raw)) return raw;
  if (LEGACY_SETTINGS_CAT[raw]) return LEGACY_SETTINGS_CAT[raw];
  return SETTINGS_CAT.GENERAL;
}

export const settingsCat = ref(SETTINGS_CAT.GENERAL);

/** App 壳 ConfigProvider / i18n / 日志换行 / 功能按钮数 */
export const locale = ref("zh-CN");
export const colorThemeId = ref("teal");
export const logWrap = ref(true);
export const actionButtonCount = ref(5);

let settingsSaveTimer = null;

// TODO(fe-theme-apply): applyAppearance 已迁出 App.vue — DESIGN §12.2
export function applyAppearance(next) {
  const g = next?.general || emptySettings().general;
  colorThemeId.value = g.theme || "teal";
  locale.value = g.locale || "zh-CN";
  logWrap.value = g.logWrap !== false;
  actionButtonCount.value = clampActionButtonCount(g.actionButtonCount);
  applyThemeToDocument(colorThemeId.value);
  document.documentElement.lang = locale.value;
}

export function loadPreviewSettings() {
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (!raw) {
      const created = emptySettings();
      localStorage.setItem(SETTINGS_KEY, JSON.stringify(created));
      return created;
    }
    return normalizeSettings(JSON.parse(raw));
  } catch {
    return emptySettings();
  }
}

/**
 * 持久化设置。
 * @param {object} [next]
 * @param {{ notify?: boolean }} [options] notify=true 时弹出成功/失败提示
 */
export async function persistSettings(next = settings.value, options = {}) {
  const { notify = false } = options;
  const t = createTranslator(locale);
  saving.value = true;
  try {
    const payload = normalizeSettings(next);
    settings.value = payload;
    applyAppearance(payload);
    if (previewMode.value) {
      localStorage.setItem(SETTINGS_KEY, JSON.stringify(payload));
    } else {
      await saveRuntimeSettings(payload);
    }
    if (notify) message.success(t("settingsSaveOk"));
  } catch (error) {
    if (notify) {
      message.error(t("settingsSaveFail", { error: String(error) }));
    }
    throw error;
  } finally {
    saving.value = false;
  }
}

export function schedulePersistSettings() {
  clearTimeout(settingsSaveTimer);
  settingsSaveTimer = setTimeout(async () => {
    try {
      await persistSettings(settings.value, { notify: true });
    } catch {
      /* 已在 persistSettings 中提示 */
    }
  }, 280);
}

export function patchSettings(updater) {
  const next = typeof updater === "function" ? updater(settings.value) : updater;
  settings.value = normalizeSettings(next);
  schedulePersistSettings();
}
