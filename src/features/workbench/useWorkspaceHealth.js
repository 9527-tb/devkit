/**
 * 工作区环境体检：扫描/切换根目录后刷新。
 * 对应 docs/DEV_PRODUCTIVITY_DESIGN.md ②
 */
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { previewMode, settings } from "../../stores/settings.js";

export function useWorkspaceHealth(getRoot, getProjects) {
  const healthReport = ref(null);
  const healthLoading = ref(false);

  async function refreshHealth() {
    const root = getRoot();
    const projects = getProjects();
    if (!root || previewMode.value) {
      healthReport.value = null;
      return;
    }
    if (settings.value.general?.healthCheckOnScan === false) {
      healthReport.value = null;
      return;
    }
    healthLoading.value = true;
    try {
      healthReport.value = await invoke("workspace_health_check", { root, projects });
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

  function projectHealthItems(projectKey) {
    const items = healthReport.value?.items || [];
    if (!projectKey) return [];
    return items.filter((item) => item.projectKey === projectKey);
  }

  const warnCount = () => {
    const items = healthReport.value?.items || [];
    return items.filter((i) => i.level === "warn" || i.level === "error").length;
  };

  return {
    healthReport,
    healthLoading,
    refreshHealth,
    projectHealthItems,
    warnCount,
  };
}

/** 将后端 action 映射为 settingsCat 或 tools 深链 */
export function resolveHealthNavigation(action) {
  if (!action) return null;
  const a = String(action);
  if (a === "tools.ports") return { type: "tools", toolId: "ports" };
  if (a.startsWith("settings.")) {
    const cat = a.replace(/^settings\./, "");
    return { type: "settings", cat };
  }
  return null;
}
