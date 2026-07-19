/**
 * 工作区状态条：环境体检 + Git（供标题栏与工作台共用）。
 */

import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { previewMode, settings } from "./settings.js";
import { root, lastScannedProjects } from "./workspace.js";

export const healthReport = ref(null);
export const healthLoading = ref(false);
export const gitStatus = ref(null);

export async function refreshWorkspaceHealth() {
  if (!root.value || previewMode.value) {
    healthReport.value = null;
    return;
  }
  if (settings.value.general?.healthCheckOnScan === false) {
    healthReport.value = null;
    return;
  }
  healthLoading.value = true;
  try {
    healthReport.value = await invoke("workspace_health_check", {
      root: root.value,
      projects: lastScannedProjects.value || [],
    });
  } catch (error) {
    console.error(error);
    healthReport.value = {
      ok: false,
      items: [
        {
          id: "healthError",
          level: "error",
          message: String(error),
        },
      ],
    };
  } finally {
    healthLoading.value = false;
  }
}

export async function refreshWorkspaceGit() {
  if (!root.value || previewMode.value) {
    gitStatus.value = null;
    return;
  }
  try {
    gitStatus.value = await invoke("workspace_git_status", { root: root.value });
  } catch (error) {
    console.error(error);
    gitStatus.value = null;
  }
}

export function projectHealthItems(projectKey) {
  const items = healthReport.value?.items || [];
  if (!projectKey) return [];
  return items.filter((item) => item.projectKey === projectKey);
}
