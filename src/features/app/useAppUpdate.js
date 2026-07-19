/**
 * 应用内自动更新：检查 → 下载安装 → 重启。
 *
 * 依赖 @tauri-apps/plugin-updater（清单 URL 见 tauri.conf.json endpoints）
 * 与 plugin-process（relaunch）。更新包经 minisign 验签，失败则拒绝安装。
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
/** 检查更新 HTTP 超时（毫秒）；未设置时底层客户端可能长时间无响应 */
const CHECK_TIMEOUT_MS = 15_000;
/** 开发模式不静默检查（本地 endpoint 常不可用） */
const IS_DEV = !!import.meta.env.DEV;

export function useAppUpdate() {
  const updateOpen = ref(false);
  const updateInfo = ref(null);
  const downloading = ref(false);
  const downloadPercent = ref(0);
  const runningCount = ref(0);
  const checking = ref(false);
  const appVersion = ref("");
  /** 发现可升级版本后保持 true（点「稍后」也不清除，用于齿轮 / 检查更新红点） */
  const updateAvailable = ref(false);

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
  async function refreshRunningCount() {
    try {
      runningCount.value = Number(await invokeSafe("count_running_processes")) || 0;
    } catch {
      runningCount.value = 0;
    }
  }

  async function checkForUpdate(opts = {}) {
    const silent = !!opts.silent;
    if (!isTauri || previewMode.value) {
      if (!silent) message.info(t("previewNeedTauriShort"));
      return false;
    }
    if (checking.value || downloading.value) return false;

    // 已发现更新时，手动点「检查更新」直接再次打开对话框
    if (!silent && pendingUpdate && updateAvailable.value) {
      await refreshRunningCount();
      updateOpen.value = true;
      return true;
    }

    checking.value = true;
    try {
      const update = await check({ timeout: CHECK_TIMEOUT_MS });
      if (!update) {
        updateAvailable.value = false;
        pendingUpdate = null;
        updateInfo.value = null;
        if (!silent) message.success(t("updateUpToDate"));
        return false;
      }
      pendingUpdate = update;
      updateAvailable.value = true;
      updateInfo.value = {
        version: update.version,
        notes: update.body || "",
        date: update.date || "",
      };
      await refreshRunningCount();
      // 发现更新：亮红点；启动静默检查与手动检查都会弹出对话框
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
    if (/network|fetch|timed out|timeout|time\s*out|dns|error sending request/i.test(s)) {
      return t("updateCheckNetwork");
    }
    return s;
  }

  function dismissUpdate() {
    if (downloading.value) return;
    updateOpen.value = false;
    downloadPercent.value = 0;
    // 保留 pendingUpdate / updateAvailable，红点继续提示可升级
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
    updateAvailable,
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
