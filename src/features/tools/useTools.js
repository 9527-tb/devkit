/**
 * 工具页开关。
 */

import { toolsPage, toolsCat } from "../../stores/tools.js";
import { settingsPage } from "../../stores/settings.js";
import { defaultToolId } from "./registry.js";

export function useTools() {
  function openTools(toolId) {
    settingsPage.value = false;
    toolsPage.value = true;
    toolsCat.value = toolId || defaultToolId();
  }

  function closeTools() {
    toolsPage.value = false;
  }

  return {
    toolsPage,
    toolsCat,
    openTools,
    closeTools,
  };
}
