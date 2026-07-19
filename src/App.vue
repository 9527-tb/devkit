<script setup>
/**
 * App 壳：ConfigProvider + InitModal + Settings / Workbench 视图切换。
 * 对应 DESIGN.md §12.2
 */
// DONE(fe-app-shell): App 仅 ConfigProvider + view 切换

import { computed, onMounted, onBeforeUnmount, provide } from "vue";
import { getTheme, BORDER_RADIUS_PX } from "./themes/index.js";
import {
  locale,
  colorThemeId,
  settingsPage,
} from "./stores/settings.js";
import { toolsPage } from "./stores/tools.js";
import { useSettings, watchLogWrapSync } from "./features/settings/useSettings.js";
import { useTools } from "./features/tools/useTools.js";
import { useFirstInit } from "./features/init/useFirstInit.js";
import { useAppQuit } from "./features/app/useAppQuit.js";
import { useAppUpdate } from "./features/app/useAppUpdate.js";
import InitModal from "./features/init/InitModal.vue";
import QuitConfirm from "./features/app/QuitConfirm.vue";
import UpdateDialog from "./features/app/UpdateDialog.vue";
import AppTitlebar from "./features/app/AppTitlebar.vue";
import SettingsView from "./views/SettingsView.vue";
import ToolsView from "./views/ToolsView.vue";
import WorkbenchView from "./views/WorkbenchView.vue";
import { createTranslator } from "./i18n/index.js";
import "./features/workbench/workbench.css";

const t = createTranslator(locale);
useSettings();
useTools();
const { bootstrap: bootstrapSettings } = useFirstInit();
const {
  quitOpen,
  quitRunningCount,
  quitLoading,
  cancelQuit,
  stopAndQuit,
  quitWithoutStop,
} = useAppQuit();
const {
  updateOpen,
  updateInfo,
  updateAvailable,
  downloading,
  downloadPercent,
  runningCount: updateRunningCount,
  checking: updateChecking,
  appVersion,
  checkForUpdate,
  dismissUpdate,
  installUpdate,
  scheduleStartupCheck,
} = useAppUpdate();
provide("appUpdate", {
  checkForUpdate,
  checking: updateChecking,
  appVersion,
  updateAvailable,
});
watchLogWrapSync();

const theme = computed(() => {
  const pack = getTheme(colorThemeId.value);
  return {
    token: {
      colorPrimary: pack.primary,
      borderRadius: BORDER_RADIUS_PX,
      fontSize: 13,
      fontFamily: '"IBM Plex Sans", "Noto Sans SC", system-ui, sans-serif',
    },
  };
});

function suppressSystemContextMenu(event) {
  event.preventDefault();
}

onMounted(async () => {
  document.addEventListener("contextmenu", suppressSystemContextMenu);
  await bootstrapSettings();
  scheduleStartupCheck();
});

onBeforeUnmount(() => {
  document.removeEventListener("contextmenu", suppressSystemContextMenu);
});
</script>

<template>
  <a-config-provider :theme="theme">
    <div class="app-shell">
      <InitModal :primary-color="theme.token.colorPrimary" />
      <AppTitlebar :t="t" />
      <div class="app-main">
        <SettingsView v-if="settingsPage" />
        <ToolsView v-else-if="toolsPage" />
        <WorkbenchView v-else :t="t" />
      </div>
      <QuitConfirm
        v-model:open="quitOpen"
        :running-count="quitRunningCount"
        :loading="quitLoading"
        @cancel="cancelQuit"
        @stop-and-quit="stopAndQuit"
        @quit-without-stop="quitWithoutStop"
      />
      <UpdateDialog
        v-model:open="updateOpen"
        :version="updateInfo?.version || ''"
        :notes="updateInfo?.notes || ''"
        :running-count="updateRunningCount"
        :downloading="downloading"
        :download-percent="downloadPercent"
        @later="dismissUpdate"
        @install="installUpdate"
      />
    </div>
  </a-config-provider>
</template>

<style>
@import url("https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600;700&family=Noto+Sans+SC:wght@400;500;700&display=swap");

:root {
  --radius: 3px;
  color-scheme: light;
  font-family: "IBM Plex Sans", "Noto Sans SC", system-ui, sans-serif;
  background: var(--dk-app-bg);
  color: var(--dk-text);
}

* { box-sizing: border-box; }
html, body, #app { margin: 0; min-height: 100%; height: 100%; }
body { background: var(--dk-app-bg); }

html,
body,
#app,
.app-shell {
  -webkit-user-select: none;
  user-select: none;
}

/* 输入与日志区仍可复制 */
input,
textarea,
select,
[contenteditable="true"],
.console-log,
.console-body,
.ant-input,
.ant-input-affix-wrapper,
.ant-select-selection-search-input {
  -webkit-user-select: text;
  user-select: text;
}

.app-shell {
  height: 100%;
  min-height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background:
    radial-gradient(circle at top left, var(--dk-glow-a), transparent 34%),
    radial-gradient(circle at top right, var(--dk-glow-b), transparent 28%),
    linear-gradient(180deg, var(--dk-app-bg), var(--dk-app-bg-2));
}

.app-main {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>
