<script setup>
/**
 * App 壳：ConfigProvider + 标题栏 + 左侧轨 + 视图切换（对齐 prototype）。
 */
import { computed, onMounted, onBeforeUnmount, provide } from "vue";
import { getTheme, BORDER_RADIUS_PX } from "./themes/index.js";
import {
  locale,
  colorThemeId,
} from "./stores/settings.js";
import { appMode } from "./stores/appNav.js";
import { useSettings, watchLogWrapSync } from "./features/settings/useSettings.js";
import { useTools } from "./features/tools/useTools.js";
import { useFirstInit } from "./features/init/useFirstInit.js";
import { useAppQuit } from "./features/app/useAppQuit.js";
import { useAppUpdate } from "./features/app/useAppUpdate.js";
import { useWorkbench } from "./features/workbench/useWorkbench.js";
import InitModal from "./features/init/InitModal.vue";
import QuitConfirm from "./features/app/QuitConfirm.vue";
import UpdateDialog from "./features/app/UpdateDialog.vue";
import AppTitlebar from "./features/app/AppTitlebar.vue";
import AppRail from "./features/app/AppRail.vue";
import SettingsView from "./views/SettingsView.vue";
import ToolsView from "./views/ToolsView.vue";
import WorkbenchView from "./views/WorkbenchView.vue";
import PlansView from "./views/PlansView.vue";
import { createTranslator } from "./i18n/index.js";
import "./features/workbench/workbench.css";

const t = createTranslator(locale);
useSettings();
useTools();
const workbench = useWorkbench(t);
provide("workbench", workbench);
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

const mode = computed(() => appMode.value);

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
      <div class="app-body">
        <AppRail :t="t" />
        <div class="app-main">
          <!-- 工作台常驻（隐藏时仍 provide 编排状态）；其余模式互斥 -->
          <div v-show="mode === 'workbench'" class="app-view">
            <WorkbenchView :t="t" />
          </div>
          <div v-show="mode === 'plans'" class="app-view">
            <PlansView :t="t" />
          </div>
          <div v-if="mode === 'tools'" class="app-view">
            <ToolsView />
          </div>
          <div v-if="mode === 'settings'" class="app-view">
            <SettingsView />
          </div>
        </div>
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
  --gap-sm: 2px;
  --gap-lg: 8px;
  color-scheme: light;
  font-family: "IBM Plex Sans", "Noto Sans SC", system-ui, sans-serif;
  font-size: 13px;
  background: var(--dk-app-bg);
  color: var(--dk-text);
}

* { box-sizing: border-box; }
html, body, #app { margin: 0; min-height: 100%; height: 100%; }
body { background: var(--dk-app-bg); font-size: 13px; }

html,
body,
#app,
.app-shell {
  -webkit-user-select: none;
  user-select: none;
}

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

.app-body {
  flex: 1;
  min-height: 0;
  display: flex;
  overflow: hidden;
}

.app-main {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}

.app-view {
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.app-view > * {
  flex: 1;
  min-width: 0;
  min-height: 0;
}

/* v-show 隐藏时勿占 flex 空间 */
.app-main > .app-view[style*="display: none"] {
  display: none !important;
  flex: none !important;
}
</style>
