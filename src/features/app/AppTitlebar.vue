<!--
  全局自定义标题栏：
  - 居中会话芯片；右侧体检 / Git / 版本三个图标
  - 主导航改由 AppRail；无品牌文案、无设置按钮
-->
<script setup>
import { computed, inject, onMounted, onBeforeUnmount, ref } from "vue";
import { TagOutlined } from "@antdv-next/icons";
import { getCurrentWindow } from "@tauri-apps/api/window";
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

const { openSettings } = useSettings();
const { openTools } = useTools();

const appUpdate = inject("appUpdate", null);
const updateAvailable = computed(() => !!appUpdate?.updateAvailable?.value);
const updateChecking = computed(() => !!appUpdate?.checking?.value);
const appVersion = computed(() => appUpdate?.appVersion?.value || "—");

const showWinControls = useCustomWindowControls;
const controlsVariant = isLinux ? "linux" : "windows";

const macTrafficInset = ref(isMac);

function onCheckUpdate() {
  appUpdate?.checkForUpdate?.({ silent: false });
}

function onHealthHandle(payload) {
  const target = resolveHealthNavigation(payload?.action);
  if (!target) return;
  if (target.type === "settings") {
    openSettings(target.cat);
    return;
  }
  if (target.type === "tools") {
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
            <GitStatusButton :t="t" :status="gitStatus" variant="icon" />
          </template>
          <a-badge :dot="updateAvailable" :offset="[-4, 4]">
            <button
              type="button"
              class="title-icon-btn"
              :class="{ loading: updateChecking }"
              :title="`${t('checkForUpdate')} · v${appVersion}`"
              :disabled="updateChecking"
              @click="onCheckUpdate"
            >
              <TagOutlined />
            </button>
          </a-badge>
        </div>
        <WindowControls v-if="showWinControls" :variant="controlsVariant" />
      </div>
    </div>
  </header>
</template>

<style scoped>
.app-titlebar {
  --titlebar-h: 34px;
  --titlebar-control-h: 26px;
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

.app-titlebar.is-mac {
  --titlebar-h: 38px;
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
  grid-template-columns: 1fr auto 1fr;
  align-items: center;
  column-gap: 8px;
  padding: 0 8px;
  pointer-events: none;
  box-sizing: border-box;
}

.app-titlebar.is-win-controls .app-titlebar-inner {
  padding-right: 0;
}

.app-titlebar-right,
.app-titlebar-actions,
.app-titlebar-center,
.title-icon-btn {
  pointer-events: auto;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.app-titlebar-center {
  grid-column: 2;
  justify-self: center;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  max-width: min(480px, 52vw);
  height: var(--titlebar-control-h);
}

.app-titlebar-right {
  grid-column: 3;
  justify-self: end;
  display: flex;
  align-items: center;
  gap: 2px;
  min-width: 0;
  height: var(--titlebar-control-h);
  z-index: 2;
}

.app-titlebar.is-win-controls .app-titlebar-right {
  align-self: stretch;
  height: 100%;
}

.app-titlebar-actions {
  display: flex;
  align-items: center;
  gap: 0;
  height: var(--titlebar-control-h);
  padding-right: 2px;
}

.app-titlebar-actions :deep(.ant-badge) {
  display: inline-flex;
  align-items: center;
  height: 100%;
  line-height: 1;
}

.app-titlebar-actions :deep(.health-icon-btn),
.app-titlebar-actions :deep(.git-icon-btn) {
  width: 26px !important;
  height: 26px !important;
  min-width: 26px !important;
}

.title-icon-btn {
  width: 26px;
  height: 26px;
  margin: 0;
  padding: 0;
  border: 0;
  border-radius: var(--radius, 3px);
  background: transparent;
  color: var(--ink-soft);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
}
.title-icon-btn:hover:not(:disabled) {
  background: var(--teal-soft);
  color: var(--teal);
}
.title-icon-btn:disabled {
  opacity: 0.55;
  cursor: default;
}
.title-icon-btn.loading {
  opacity: 0.7;
}
</style>
