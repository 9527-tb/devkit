/**
 * 应用内更新：check / downloadAndInstall / relaunch。
 * 基于 @tauri-apps/plugin-updater + plugin-process。
 */

import { onMounted, onBeforeUnmount, ref } from "vue";
import { message } from "antdv-next";
import { getVersion } from "@tauri-apps/api/app";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { invokeSafe, isTauri } from "../../api/tauri.js";
import { createTranslator } from "../../i18n/index.js";
import { locale, previewMode, settings } from "../../stores/settings.js";

const t = createTranslator(locale);

const STARTUP_DELAY_MS = 3000;
/** Vite / tauri dev：静默检查无意义（endpoint 常 404），仅手动检查 */
const IS_DEV = !!import.meta.env.DEV;

export function useAppUpdate() {
  const updateOpen = ref(false);
  const updateInfo = ref(null);
  const downloading = ref(false);
  const downloadPercent = ref(0);
  const runningCount = ref(0);
  const checking = ref(false);
  const appVersion = ref("");

  /** @type {import('@tauri-apps/plugin-updater').Update | null} */
  let pendingUpdate = null;
  let startupTimer = null;

  async function refreshAppVersion() {
    if (!isTauri || previewMode.value) {
      appVersion.value = "";
      return;
    }
    try {
      appVersion.value = await getVersion();
    } catch {
      appVersion.value = "";
    }
  }

  /**
   * @param {{ silent?: boolean }} [opts]
   * @returns {Promise<boolean>} 是否发现更新
   */
  async function checkForUpdate(opts = {}) {
    const silent = !!opts.silent;
    if (!isTauri || previewMode.value) {
      if (!silent) message.info(t("previewNeedTauriShort"));
      return false;
    }
    if (checking.value || downloading.value) return false;
    checking.value = true;
    try {
      const update = await check();
      if (!update) {
        if (!silent) message.success(t("updateUpToDate"));
        return false;
      }
      pendingUpdate = update;
      updateInfo.value = {
        version: update.version,
        notes: update.body || "",
        date: update.date || "",
      };
      try {
        runningCount.value = Number(await invokeSafe("count_running_processes")) || 0;
      } catch {
        runningCount.value = 0;
      }
      updateOpen.value = true;
      return true;
    } catch (error) {
      const raw = String(error || "");
      const friendly = friendlyUpdateError(raw);
      if (!silent) {
        message.error(t("updateCheckFail", { error: friendly }));
      } else {
        console.warn("[update] silent check failed", raw);
      }
      return false;
    } finally {
      checking.value = false;
    }
  }

  function friendlyUpdateError(raw) {
    const s = String(raw || "");
    if (/404|Not Found|Could not fetch/i.test(s)) {
      return t("updateCheckNoRelease");
    }
    if (/signature|verify|pubkey|minisign/i.test(s)) {
      return t("updateCheckBadSignature");
    }
    if (/network|fetch|timed out|timeout|dns/i.test(s)) {
      return t("updateCheckNetwork");
    }
    return s;
  }

  function dismissUpdate() {
    if (downloading.value) return;
    updateOpen.value = false;
    updateInfo.value = null;
    pendingUpdate = null;
    downloadPercent.value = 0;
  }

  async function installUpdate() {
    if (!pendingUpdate || downloading.value) return;
    downloading.value = true;
    downloadPercent.value = 0;
    try {
      if (runningCount.value > 0) {
        await invokeSafe("stop_all_processes");
        runningCount.value = 0;
      }

      let downloaded = 0;
      let contentLength = 0;
      await pendingUpdate.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            contentLength = Number(event.data.contentLength) || 0;
            downloaded = 0;
            downloadPercent.value = 0;
            break;
          case "Progress": {
            downloaded += Number(event.data.chunkLength) || 0;
            if (contentLength > 0) {
              downloadPercent.value = Math.min(
                99,
                Math.round((downloaded / contentLength) * 100),
              );
            }
            break;
          }
          case "Finished":
            downloadPercent.value = 100;
            break;
          default:
            break;
        }
      });

      message.success(t("updateInstalled"));
      await relaunch();
    } catch (error) {
      message.error(t("updateInstallFail", { error: String(error) }));
      downloading.value = false;
    }
  }

  function scheduleStartupCheck() {
    if (!isTauri || previewMode.value || IS_DEV) return;
    if (startupTimer) clearTimeout(startupTimer);
    startupTimer = setTimeout(() => {
      startupTimer = null;
      if (settings.value.general?.autoCheckUpdate === false) return;
      checkForUpdate({ silent: true });
    }, STARTUP_DELAY_MS);
  }

  onMounted(async () => {
    await refreshAppVersion();
  });

  onBeforeUnmount(() => {
    if (startupTimer) clearTimeout(startupTimer);
  });

  return {
    updateOpen,
    updateInfo,
    downloading,
    downloadPercent,
    runningCount,
    checking,
    appVersion,
    checkForUpdate,
    dismissUpdate,
    installUpdate,
    refreshAppVersion,
    scheduleStartupCheck,
  };
}
