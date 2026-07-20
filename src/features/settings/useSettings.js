/**
 * 设置页 composable：加载、补丁保存、应用外观。
 * 依赖：stores/settings、api/settings、themes。
 * 对应 DESIGN.md §12.2 useSettings
 */

import { watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { message } from "antdv-next";
import { applyThemeToDocument } from "../../themes/index.js";
import { createTranslator } from "../../i18n/index.js";
import {
  settings,
  settingsPage,
  settingsCat,
  settingsLoading,
  locale,
  colorThemeId,
  logWrap,
  actionButtonCount,
  previewMode,
  persistSettings,
  schedulePersistSettings,
  applyAppearance,
  loadPreviewSettings,
  SETTINGS_CAT,
  normalizeSettingsCat,
  normalizeSettings,
  normalizeNodePackageManager,
  clampActionButtonCount,
  clampOrchestrationLimit,
  entryKey,
  patchSettings,
} from "../../stores/settings.js";
import { toolsPage, toolsQuery } from "../../stores/tools.js";
import { plansPage } from "../../stores/appNav.js";

const t = createTranslator(locale);
import {
  loadRuntimeSettings,
  validateRuntimePath,
} from "../../api/settings.js";

// TODO(fe-settings-store): DONE — load / patch / applyAppearance — DESIGN §12.2

export function useSettings() {
  async function load() {
    settingsLoading.value = true;
    try {
      if (previewMode.value) {
        settings.value = loadPreviewSettings();
      } else {
        settings.value = normalizeSettings(await loadRuntimeSettings());
      }
      applyAppearance(settings.value);
    } catch (error) {
      message.error(String(error));
    } finally {
      settingsLoading.value = false;
    }
  }

  async function patch(updater) {
    patchSettings(updater);
  }

  async function openSettings(cat) {
    toolsPage.value = false;
    plansPage.value = false;
    settingsPage.value = true;
    settingsCat.value = normalizeSettingsCat(cat || SETTINGS_CAT.GENERAL);
    await load();
  }

  function closeSettings() {
    settingsPage.value = false;
  }

  function onThemeChange(id) {
    settings.value = {
      ...settings.value,
      general: { ...settings.value.general, theme: id },
    };
    applyThemeToDocument(id);
    colorThemeId.value = id;
    schedulePersistSettings();
  }

  function onLocaleChange(id) {
    settings.value = {
      ...settings.value,
      general: { ...settings.value.general, locale: id },
    };
    locale.value = id;
    document.documentElement.lang = id;
    schedulePersistSettings();
  }

  function onLogWrapSettingChange(checked) {
    settings.value = {
      ...settings.value,
      general: { ...settings.value.general, logWrap: !!checked },
    };
    logWrap.value = !!checked;
    schedulePersistSettings();
  }

  function onNotifyOnTaskDoneChange(checked) {
    settings.value = {
      ...settings.value,
      general: { ...settings.value.general, notifyOnTaskDone: !!checked },
    };
    schedulePersistSettings();
  }

  function onHealthCheckOnScanChange(checked) {
    settings.value = {
      ...settings.value,
      general: { ...settings.value.general, healthCheckOnScan: !!checked },
    };
    schedulePersistSettings();
  }

  function onPreferWorkspaceConfigChange(checked) {
    settings.value = {
      ...settings.value,
      general: { ...settings.value.general, preferWorkspaceConfig: !!checked },
    };
    schedulePersistSettings();
  }

  function onEditorCommandChange(value) {
    settings.value = {
      ...settings.value,
      general: { ...settings.value.general, editorCommand: String(value || "").trim() },
    };
    schedulePersistSettings();
  }

  function onTerminalAppChange(value) {
    settings.value = {
      ...settings.value,
      general: { ...settings.value.general, terminalApp: String(value || "").trim() },
    };
    schedulePersistSettings();
  }

  function onActionButtonCountChange(value) {
    const count = clampActionButtonCount(value);
    settings.value = {
      ...settings.value,
      general: { ...settings.value.general, actionButtonCount: count },
    };
    actionButtonCount.value = count;
    schedulePersistSettings();
  }

  function onMaxWorkspaceRootsChange(value) {
    settings.value = {
      ...settings.value,
      general: {
        ...settings.value.general,
        maxWorkspaceRoots: clampOrchestrationLimit(value, 10),
      },
    };
    schedulePersistSettings();
  }

  function onMaxParallelSpawnsChange(value) {
    settings.value = {
      ...settings.value,
      general: {
        ...settings.value.general,
        maxParallelSpawns: clampOrchestrationLimit(value, 10),
      },
    };
    schedulePersistSettings();
  }

  function onMavenHomeInput(value) {
    settings.value = {
      ...settings.value,
      java: { ...settings.value.java, mavenHome: value },
    };
    schedulePersistSettings();
  }

  function onNodePackageManagerChange(value) {
    settings.value = {
      ...settings.value,
      node: {
        ...settings.value.node,
        packageManager: normalizeNodePackageManager(value),
      },
    };
    schedulePersistSettings();
  }

  function onLaunchAtLoginChange(checked) {
    settings.value = {
      ...settings.value,
      general: { ...settings.value.general, launchAtLogin: !!checked },
    };
    schedulePersistSettings();
  }

  function onCloseToTrayChange(checked) {
    settings.value = {
      ...settings.value,
      general: { ...settings.value.general, closeToTray: !!checked },
    };
    schedulePersistSettings();
  }

  function onAutoCheckUpdateChange(checked) {
    settings.value = {
      ...settings.value,
      general: { ...settings.value.general, autoCheckUpdate: !!checked },
    };
    schedulePersistSettings();
  }

  async function addRuntimePath(kind) {
    if (previewMode.value) {
      message.info(t("previewCannotPickDir"));
      return;
    }
    const picked = await open({
      directory: true,
      multiple: false,
      title: kind === "jdk" ? t("pickJdkDir") : t("pickNodeDir"),
    });
    if (!picked) return;
    try {
      const entry = await validateRuntimePath(kind, picked);
      if (kind === "jdk") {
        const exists = settings.value.java.jdks.some((e) => e.path === entry.path);
        if (exists) return message.warning(t("jdkPathExists"));
        settings.value = {
          ...settings.value,
          java: {
            ...settings.value.java,
            jdks: [...settings.value.java.jdks, entry],
          },
        };
      } else {
        const exists = settings.value.node.nodes.some((e) => e.path === entry.path);
        if (exists) return message.warning(t("nodePathExists"));
        settings.value = {
          ...settings.value,
          node: {
            ...settings.value.node,
            nodes: [...settings.value.node.nodes, entry],
          },
        };
      }
      await persistSettings(settings.value, { notify: false });
      message.success(t("runtimeAdded", { label: entry.label || entry.version }));
    } catch (error) {
      message.error(t("settingsSaveFail", { error: String(error) }));
    }
  }

  async function pickMavenHome() {
    if (previewMode.value) {
      message.info(t("previewCannotPickDir"));
      return;
    }
    const picked = await open({
      directory: true,
      multiple: false,
      title: t("pickMavenHome"),
    });
    if (!picked) return;
    settings.value = {
      ...settings.value,
      java: { ...settings.value.java, mavenHome: picked },
    };
    try {
      await persistSettings(settings.value, { notify: true });
    } catch {
      /* 已提示 */
    }
  }

  function removeRuntime(kind, id) {
    if (kind === "jdk") {
      settings.value = {
        ...settings.value,
        java: {
          ...settings.value.java,
          jdks: settings.value.java.jdks.filter((e) => entryKey(e) !== id),
        },
      };
    } else {
      settings.value = {
        ...settings.value,
        node: {
          ...settings.value.node,
          nodes: settings.value.node.nodes.filter((e) => entryKey(e) !== id),
        },
      };
    }
    schedulePersistSettings();
  }

  return {
    settings,
    settingsPage,
    settingsCat,
    settingsLoading,
    load,
    patch,
    applyAppearance,
    openSettings,
    closeSettings,
    onThemeChange,
    onLocaleChange,
    onLogWrapSettingChange,
    onNotifyOnTaskDoneChange,
    onHealthCheckOnScanChange,
    onPreferWorkspaceConfigChange,
    onEditorCommandChange,
    onTerminalAppChange,
    onActionButtonCountChange,
    onMaxWorkspaceRootsChange,
    onMaxParallelSpawnsChange,
    onMavenHomeInput,
    onNodePackageManagerChange,
    onLaunchAtLoginChange,
    onCloseToTrayChange,
    onAutoCheckUpdateChange,
    addRuntimePath,
    pickMavenHome,
    removeRuntime,
  };
}

/** 设置页打开时同步 logWrap 变更（与 App.vue 原行为一致） */
export function watchLogWrapSync() {
  watch(logWrap, (v) => {
    if (!settingsPage.value) return;
    if (settings.value.general.logWrap === v) return;
    settings.value = {
      ...settings.value,
      general: { ...settings.value.general, logWrap: v },
    };
    schedulePersistSettings();
  });
}
