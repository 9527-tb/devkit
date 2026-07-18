/**
 * 首次初始化编排 composable：检测配置是否存在并驱动 InitModal。
 * 依赖：stores/init、stores/settings、api/settings。
 * 对应 DESIGN.md §6.1 / §12.2
 */

import { message } from "antdv-next";
import { createTranslator } from "../../i18n/index.js";
import {
  settings,
  SETTINGS_KEY,
  previewMode,
  emptySettings,
  normalizeSettings,
  applyAppearance,
  loadPreviewSettings,
  locale,
} from "../../stores/settings.js";
import {
  visible,
  inProgress,
  detail,
  resetInitUi,
  applyInitProgress,
} from "../../stores/init.js";
import {
  runtimeSettingsExists,
  loadRuntimeSettings,
  initializeRuntimeSettings,
  listenInitProgress,
} from "../../api/settings.js";

// TODO(fe-init-modal): DONE — bootstrap（exists → initialize / load）— DESIGN §6.1

const t = createTranslator(locale);

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function runPreviewFirstInit() {
  const steps = [
    { step: "prepare", percent: 8 },
    { step: "scanJdk", percent: 40, found: 2 },
    { step: "scanNode", percent: 65, found: 2 },
    { step: "scanMaven", percent: 84, found: 1, detail: "/opt/homebrew/opt/maven/libexec" },
    { step: "writeConfig", percent: 92 },
    { step: "done", percent: 100, done: true },
  ];
  for (const item of steps) {
    applyInitProgress(item);
    await sleep(item.done ? 400 : 520);
  }
  const created = emptySettings();
  created.java.jdks = [
    { id: "jdk:21:preview", version: "21", path: "/Library/Java/JavaVirtualMachines/preview-21", label: "JDK 21", manual: false },
    { id: "jdk:17:preview", version: "17", path: "/Library/Java/JavaVirtualMachines/preview-17", label: "JDK 17", manual: false },
  ];
  created.node.nodes = [
    { id: "node:20:preview", version: "20", path: "/Users/preview/.nvm/versions/node/v20", label: "Node 20", manual: false },
  ];
  created.java.mavenHome = "/opt/homebrew/opt/maven/libexec";
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(created));
  return created;
}

async function runFirstInitFlow() {
  resetInitUi();
  inProgress.value = true;
  visible.value = true;
  detail.value = t("initWait");

  let unlisten = null;
  try {
    if (!previewMode.value) {
      // TODO(rs-init-progress): init-progress 事件驱动进度 UI
      unlisten = await listenInitProgress((event) => {
        applyInitProgress(event.payload);
      });
      const payload = await initializeRuntimeSettings();
      settings.value = normalizeSettings(payload);
    } else {
      settings.value = normalizeSettings(await runPreviewFirstInit());
    }
    applyAppearance(settings.value);
    applyInitProgress({ step: "done", percent: 100, done: true });
    await sleep(700);
  } catch (error) {
    message.error(String(error));
    await sleep(900);
  } finally {
    if (typeof unlisten === "function") unlisten();
    inProgress.value = false;
    visible.value = false;
  }
}

export function useFirstInit() {
  async function bootstrap() {
    if (previewMode.value) {
      const raw = localStorage.getItem(SETTINGS_KEY);
      if (!raw) {
        await runFirstInitFlow();
        return;
      }
      settings.value = loadPreviewSettings();
      applyAppearance(settings.value);
      return;
    }

    const exists = await runtimeSettingsExists();
    if (!exists) {
      await runFirstInitFlow();
      return;
    }
    settings.value = normalizeSettings(await loadRuntimeSettings());
    applyAppearance(settings.value);
  }

  return {
    bootstrap,
    runFirstInitFlow,
  };
}
