/**
 * 应用主导航模式（对齐 prototype 左侧图标轨）。
 * 与 settingsPage / toolsPage 互斥同步，便于旧入口兼容。
 */

import { computed, ref, watch } from "vue";
import { settingsPage } from "./settings.js";
import { toolsPage } from "./tools.js";

/** @typedef {"workbench" | "plans" | "tools" | "settings"} AppMode */

/** 运行计划模式（全屏 Plans 视图） */
export const plansPage = ref(false);

export const appMode = computed(() => {
  if (settingsPage.value) return "settings";
  if (toolsPage.value) return "tools";
  if (plansPage.value) return "plans";
  return "workbench";
});

export function goWorkbench() {
  settingsPage.value = false;
  toolsPage.value = false;
  plansPage.value = false;
}

export function goPlans() {
  settingsPage.value = false;
  toolsPage.value = false;
  plansPage.value = true;
}

export function goTools() {
  settingsPage.value = false;
  plansPage.value = false;
  toolsPage.value = true;
}

export function goSettings() {
  toolsPage.value = false;
  plansPage.value = false;
  settingsPage.value = true;
}

/** 打开设置/工具时自动关掉 plans */
watch([settingsPage, toolsPage], ([s, t]) => {
  if (s || t) plansPage.value = false;
});
