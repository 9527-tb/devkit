<!--
  左侧图标轨：工作台 / 运行计划 / 工具 / 设置（对齐 prototype）。
-->
<script setup>
import { computed } from "vue";
import {
  UnorderedListOutlined,
  PlaySquareOutlined,
  AppstoreOutlined,
  SettingOutlined,
} from "@antdv-next/icons";
import { appMode, goWorkbench, goPlans } from "../../stores/appNav.js";
import { useSettings } from "../settings/useSettings.js";
import { useTools } from "../tools/useTools.js";

defineProps({
  t: { type: Function, required: true },
});

const { openSettings } = useSettings();
const { openTools } = useTools();

const mode = computed(() => appMode.value);

const items = computed(() => [
  {
    id: "workbench",
    title: "navProject",
    icon: UnorderedListOutlined,
    onClick: () => goWorkbench(),
  },
  {
    id: "plans",
    title: "runPlan",
    icon: PlaySquareOutlined,
    onClick: () => goPlans(),
  },
  {
    id: "tools",
    title: "navTools",
    icon: AppstoreOutlined,
    onClick: () => openTools(),
  },
  {
    id: "settings",
    title: "settings",
    icon: SettingOutlined,
    onClick: () => openSettings(),
    bottom: true,
  },
]);

const topItems = computed(() => items.value.filter((i) => !i.bottom));
const bottomItems = computed(() => items.value.filter((i) => i.bottom));
</script>

<template>
  <nav class="app-rail" aria-label="main">
    <button
      v-for="item in topItems"
      :key="item.id"
      type="button"
      class="app-rail-btn"
      :class="{ active: mode === item.id }"
      :title="t(item.title)"
      @click="item.onClick"
    >
      <component :is="item.icon" class="app-rail-icon" />
    </button>
    <div class="app-rail-gap" />
    <button
      v-for="item in bottomItems"
      :key="item.id"
      type="button"
      class="app-rail-btn"
      :class="{ active: mode === item.id }"
      :title="t(item.title)"
      @click="item.onClick"
    >
      <component :is="item.icon" class="app-rail-icon" />
    </button>
  </nav>
</template>

<style scoped>
.app-rail {
  flex: none;
  width: 44px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 8px 0;
  border-right: 1px solid var(--line);
  background: color-mix(in srgb, var(--panel) 88%, white);
  z-index: 40;
}
.app-rail-gap {
  flex: 1;
}
.app-rail-btn {
  width: 34px;
  height: 34px;
  margin: 0;
  padding: 0;
  border: 0;
  border-radius: var(--radius, 3px);
  background: transparent;
  color: var(--muted);
  display: grid;
  place-items: center;
  cursor: pointer;
  position: relative;
  transition: color 0.15s, background 0.15s;
}
.app-rail-btn:hover {
  color: var(--ink);
  background: var(--line-soft);
}
.app-rail-btn.active {
  color: var(--teal);
  background: var(--teal-soft);
}
.app-rail-btn.active::before {
  content: "";
  position: absolute;
  left: -5px;
  width: 3px;
  height: 14px;
  border-radius: 0 2px 2px 0;
  background: var(--teal);
}
.app-rail-icon {
  font-size: 16px;
  line-height: 1;
  display: inline-flex;
}
.app-rail-icon :deep(svg) {
  width: 16px;
  height: 16px;
  display: block;
}
</style>
