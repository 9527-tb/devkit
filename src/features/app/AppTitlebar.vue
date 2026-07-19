<!--
  全局自定义标题栏（Tauri 规范）：
  - macOS：Overlay + 原生红绿灯（左侧），内容左缩进
  - Windows/Linux：decorations:false + 右侧自定义窗口按钮
  - 标题区可拖拽；右侧操作为 type=text
-->
<script setup>
import { computed, inject, onMounted, onBeforeUnmount, ref } from "vue";
import {
  SettingOutlined,
  CloudDownloadOutlined,
  LayoutFilled,
  AppstoreFilled,
} from "@antdv-next/icons";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { settingsPage } from "../../stores/settings.js";
import { toolsPage } from "../../stores/tools.js";
import { root } from "../../stores/workspace.js";
import {
  healthReport,
  healthLoading,
  gitStatus,
  refreshWorkspaceHealth,
} from "../../stores/workspaceStatus.js";
import { isMac, isLinux, useCustomWindowControls } from "../../shared/platform.js";
import { isTauri } from "../../api/tauri.js";
import { useSettings } from "../settings/useSettings.js";
import { useTools } from "../tools/useTools.js";
import { resolveHealthNavigation } from "../workbench/useWorkspaceHealth.js";
import HealthPopover from "../workbench/HealthPopover.vue";
import WindowControls from "./WindowControls.vue";
import WorkspaceSwitcher from "./WorkspaceSwitcher.vue";
import GitStatusButton from "./GitStatusButton.vue";

defineProps({
  t: { type: Function, required: true },
});

const { openSettings, closeSettings } = useSettings();
const { openTools, closeTools } = useTools();

const appUpdate = inject("appUpdate", null);
const updateAvailable = computed(() => !!appUpdate?.updateAvailable?.value);
const updateChecking = computed(() => !!appUpdate?.checking?.value);

const showWinControls = useCustomWindowControls;
const controlsVariant = isLinux ? "linux" : "windows";

/** macOS：非全屏时为红绿灯留白（最大化仍显示红绿灯）；仅全屏时收回留白 */
const macTrafficInset = ref(isMac);

const activeNav = computed(() => {
  if (toolsPage.value) return "tools";
  if (settingsPage.value) return "settings";
  return "project";
});

function goProject() {
  closeSettings();
  closeTools();
}

function goTools() {
  closeSettings();
  openTools();
}

function onCheckUpdate() {
  appUpdate?.checkForUpdate?.({ silent: false });
}

function onOpenSettings() {
  closeTools();
  openSettings();
}

function onHealthHandle(payload) {
  const target = resolveHealthNavigation(payload?.action);
  if (!target) return;
  if (target.type === "settings") {
    closeTools();
    openSettings(target.cat);
    return;
  }
  if (target.type === "tools") {
    closeSettings();
    openTools(target.toolId, { port: payload?.port });
  }
}

let unlistenResize;
let unlistenScale;
let lastFullscreen = null;

async function refreshMacTrafficInset() {
  if (!isMac || !isTauri) {
    macTrafficInset.value = false;
    return;
  }
  try {
    const fullscreen = await getCurrentWindow().isFullscreen();
    if (fullscreen === lastFullscreen) return;
    lastFullscreen = fullscreen;
    macTrafficInset.value = !fullscreen;
  } catch {
    macTrafficInset.value = true;
  }
}

function scheduleMacTrafficInset() {
  refreshMacTrafficInset();
}

onMounted(async () => {
  await refreshMacTrafficInset();
  if (!isTauri || !isMac) return;
  try {
    const win = getCurrentWindow();
    unlistenResize = await win.onResized(() => {
      scheduleMacTrafficInset();
    });
    unlistenScale = await win.onScaleFactorChanged(() => {
      scheduleMacTrafficInset();
    });
  } catch {
    /* ignore */
  }
});

onBeforeUnmount(() => {
  if (typeof unlistenResize === "function") unlistenResize();
  if (typeof unlistenScale === "function") unlistenScale();
});
</script>

<template>
  <header
    class="app-titlebar"
    :class="{
      'is-mac': isMac,
      'is-mac-traffic-inset': isMac && macTrafficInset,
      'is-win-controls': showWinControls,
    }"
  >
    <div class="app-titlebar-drag" data-tauri-drag-region />

    <div class="app-titlebar-inner">
      <div class="app-titlebar-left">
        <nav class="app-titlebar-nav" aria-label="main">
          <button
            type="button"
            class="app-titlebar-tab"
            :class="{ active: activeNav === 'project' }"
            @click="goProject"
          >
            <LayoutFilled class="app-titlebar-tab-icon" />
            <span>{{ t("navProject") }}</span>
          </button>
          <button
            type="button"
            class="app-titlebar-tab"
            :class="{ active: activeNav === 'tools' }"
            @click="goTools"
          >
            <AppstoreFilled class="app-titlebar-tab-icon" />
            <span>{{ t("navTools") }}</span>
          </button>
        </nav>
      </div>

      <div class="app-titlebar-center">
        <WorkspaceSwitcher :t="t" />
      </div>

      <div class="app-titlebar-right">
        <div class="app-titlebar-actions">
          <template v-if="root">
            <HealthPopover
              :t="t"
              trigger="icon"
              :report="healthReport"
              :loading="healthLoading"
              @refresh="refreshWorkspaceHealth"
              @handle="onHealthHandle"
            />
            <GitStatusButton :t="t" :status="gitStatus" />
            <div class="app-titlebar-divider" aria-hidden="true" />
          </template>
          <a-badge :dot="updateAvailable" :offset="[-2, 2]">
            <a-button
              type="text"
              class="app-titlebar-icon-btn"
              :title="t('checkForUpdate')"
              :loading="updateChecking"
              @click="onCheckUpdate"
            >
              <template #icon><CloudDownloadOutlined /></template>
            </a-button>
          </a-badge>
          <a-button
            type="text"
            class="app-titlebar-icon-btn"
            :class="{ 'is-active': activeNav === 'settings' }"
            :title="t('settings')"
            @click="onOpenSettings"
          >
            <template #icon><SettingOutlined /></template>
          </a-button>
        </div>
        <WindowControls v-if="showWinControls" :variant="controlsVariant" />
      </div>
    </div>
  </header>
</template>

<style scoped>
/*
  标题栏内容始终纵向居中：
  - 底部分割用 inset box-shadow，避免 border 占高导致 flex 视觉偏上
  - 左右栏 align-self:center，Tab 高度随栏高变化
*/
.app-titlebar {
  --titlebar-h: 37px;
  --titlebar-control-h: calc(var(--titlebar-h) - 12px);
  position: relative;
  z-index: 60;
  flex: none;
  height: var(--titlebar-h);
  box-sizing: border-box;
  display: flex;
  align-items: center;
  background: color-mix(in srgb, var(--panel) 92%, transparent);
  box-shadow: inset 0 -1px 0 color-mix(in srgb, var(--line) 80%, white);
  backdrop-filter: blur(12px);
  user-select: none;
}

/* macOS Overlay：非全屏为红绿灯留白；全屏收回 */
.app-titlebar.is-mac {
  --titlebar-h: 41px;
  padding-left: 1px;
}
.app-titlebar.is-mac.is-mac-traffic-inset {
  padding-left: 78px;
}

.app-titlebar-drag {
  position: absolute;
  inset: 0;
  z-index: 0;
}

.app-titlebar-inner {
  position: relative;
  z-index: 1;
  flex: 1;
  min-width: 0;
  height: 100%;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  align-items: center;
  column-gap: 8px;
  padding: 0 6px 0 8px;
  pointer-events: none;
  box-sizing: border-box;
}

.app-titlebar.is-win-controls .app-titlebar-inner {
  padding-right: 0;
}

.app-titlebar-left,
.app-titlebar-right,
.app-titlebar-nav,
.app-titlebar-actions,
.app-titlebar-tab,
.app-titlebar-icon-btn,
.app-titlebar-center {
  pointer-events: auto;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.app-titlebar-left {
  grid-column: 1;
  justify-self: start;
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  height: var(--titlebar-control-h);
  z-index: 2;
}

.app-titlebar-right {
  grid-column: 3;
  justify-self: end;
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
  height: var(--titlebar-control-h);
  z-index: 2;
}

.app-titlebar.is-win-controls .app-titlebar-right {
  align-self: stretch;
  height: 100%;
}

.app-titlebar-nav {
  display: flex;
  align-items: center;
  gap: 2px;
  height: var(--titlebar-control-h);
}

.app-titlebar-tab {
  box-sizing: border-box;
  height: 100%;
  margin: 0;
  padding: 0 12px;
  border: 0;
  border-radius: var(--radius);
  background: transparent;
  color: var(--ink-soft);
  font: inherit;
  font-size: 13px;
  font-weight: 600;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  cursor: pointer;
}

.app-titlebar-tab > span {
  display: inline-flex;
  align-items: center;
  line-height: 1;
}

.app-titlebar-tab-icon {
  flex: none;
  width: 12px;
  height: 12px;
  font-size: 12px;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.app-titlebar-tab-icon :deep(svg) {
  width: 12px;
  height: 12px;
  display: block;
}

.app-titlebar-tab:hover {
  background: var(--nav-hover);
  color: var(--ink);
}

.app-titlebar-tab.active {
  background: var(--teal-soft);
  color: var(--teal);
}

.app-titlebar-center {
  grid-column: 2;
  justify-self: center;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  max-width: min(420px, 46vw);
  height: var(--titlebar-control-h);
}

.app-titlebar-actions {
  display: flex;
  align-items: center;
  gap: 0;
  height: var(--titlebar-control-h);
  padding-right: 2px;
}

.app-titlebar-divider {
  width: 1px;
  height: 14px;
  margin: 0 6px;
  background: color-mix(in srgb, var(--line) 90%, transparent);
  flex: none;
}

.app-titlebar-actions :deep(.ant-badge) {
  display: inline-flex;
  align-items: center;
  height: 100%;
  line-height: 1;
}

.app-titlebar-icon-btn {
  width: var(--titlebar-control-h) !important;
  height: 100% !important;
  min-width: var(--titlebar-control-h) !important;
  padding: 0 !important;
  display: inline-flex !important;
  align-items: center;
  justify-content: center;
  line-height: 1 !important;
  color: var(--ink-soft);
}

.app-titlebar-icon-btn:hover {
  color: var(--ink);
}

.app-titlebar-icon-btn.is-active {
  color: var(--teal);
  background: var(--teal-soft);
}

.app-titlebar-icon-btn :deep(.anticon) {
  line-height: 1;
  display: inline-flex;
  align-items: center;
}
</style>
