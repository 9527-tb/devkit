<!--
  Windows / Linux 自定义窗口按钮（最小化 / 最大化 / 关闭）。
  macOS 使用系统原生红绿灯，不渲染本组件。
-->
<script setup>
import { onMounted, onBeforeUnmount, ref } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { isTauri } from "../../api/tauri.js";
import { isWindows } from "../../shared/platform.js";

defineProps({
  /** windows | linux — 影响按钮视觉 */
  variant: { type: String, default: "windows" },
});

const maximized = ref(false);
let unlisten;

async function refreshMaximized() {
  if (!isTauri) return;
  try {
    maximized.value = await getCurrentWindow().isMaximized();
  } catch {
    maximized.value = false;
  }
}

async function onMinimize() {
  if (!isTauri) return;
  await getCurrentWindow().minimize();
}

async function onToggleMaximize() {
  if (!isTauri) return;
  await getCurrentWindow().toggleMaximize();
  await refreshMaximized();
}

async function onClose() {
  if (!isTauri) return;
  await getCurrentWindow().close();
}

onMounted(async () => {
  await refreshMaximized();
  if (!isTauri) return;
  try {
    unlisten = await getCurrentWindow().onResized(() => {
      refreshMaximized();
    });
  } catch {
    /* ignore */
  }
});

onBeforeUnmount(() => {
  if (typeof unlisten === "function") unlisten();
});
</script>

<template>
  <div
    class="win-controls"
    :class="[variant === 'linux' ? 'is-linux' : 'is-windows', { 'is-win': isWindows }]"
  >
    <button type="button" class="win-btn win-min" title="Minimize" @click="onMinimize">
      <svg viewBox="0 0 12 12" aria-hidden="true">
        <rect x="1" y="5.5" width="10" height="1" fill="currentColor" />
      </svg>
    </button>
    <button
      type="button"
      class="win-btn win-max"
      :title="maximized ? 'Restore' : 'Maximize'"
      @click="onToggleMaximize"
    >
      <svg v-if="!maximized" viewBox="0 0 12 12" aria-hidden="true">
        <rect
          x="1.5"
          y="1.5"
          width="9"
          height="9"
          fill="none"
          stroke="currentColor"
          stroke-width="1"
        />
      </svg>
      <svg v-else viewBox="0 0 12 12" aria-hidden="true">
        <path
          fill="none"
          stroke="currentColor"
          stroke-width="1"
          d="M3.5 3.5h6v6h-6zM2.5 4.5v6h6"
        />
      </svg>
    </button>
    <button type="button" class="win-btn win-close" title="Close" @click="onClose">
      <svg viewBox="0 0 12 12" aria-hidden="true">
        <path
          d="M2.2 2.2l7.6 7.6M9.8 2.2l-7.6 7.6"
          fill="none"
          stroke="currentColor"
          stroke-width="1.1"
        />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.win-controls {
  display: flex;
  align-items: stretch;
  align-self: stretch;
  height: 100%;
  flex: none;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.win-btn {
  appearance: none;
  margin: 0;
  padding: 0;
  border: 0;
  width: 46px;
  height: 100%;
  min-height: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  color: var(--ink-soft);
  cursor: default;
  line-height: 0;
}

.win-btn svg {
  width: 10px;
  height: 10px;
  display: block;
}

.win-btn:hover {
  background: color-mix(in srgb, var(--ink) 8%, transparent);
  color: var(--ink);
}

.win-close:hover {
  background: #e81123;
  color: #fff;
}

.is-linux .win-btn {
  width: 40px;
  border-radius: 0;
}

.is-linux .win-btn:hover {
  background: color-mix(in srgb, var(--ink) 10%, transparent);
}

.is-linux .win-close:hover {
  background: #e81123;
  color: #fff;
}
</style>
