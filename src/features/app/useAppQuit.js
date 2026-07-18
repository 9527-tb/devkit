/**
 * 应用退出确认：有运行中实例时弹三按钮对话框。
 */

import { onMounted, onBeforeUnmount, ref } from "vue";
import { message } from "antdv-next";
import { invokeSafe, listenSafe, isTauri } from "../../api/tauri.js";
import { createTranslator } from "../../i18n/index.js";
import { locale } from "../../stores/settings.js";

const t = createTranslator(locale);

export function useAppQuit() {
  const quitOpen = ref(false);
  const quitRunningCount = ref(0);
  const quitLoading = ref(false);
  let unlisten = null;
  let handling = false;

  async function handleQuitRequest() {
    if (!isTauri || handling || quitOpen.value) return;
    handling = true;
    try {
      const n = Number(await invokeSafe("count_running_processes")) || 0;
      if (n <= 0) {
        await invokeSafe("exit_app");
        return;
      }
      quitRunningCount.value = n;
      quitOpen.value = true;
    } catch (error) {
      message.error(String(error));
    } finally {
      handling = false;
    }
  }

  function cancelQuit() {
    quitOpen.value = false;
    quitLoading.value = false;
  }

  async function stopAndQuit() {
    quitLoading.value = true;
    try {
      await invokeSafe("stop_all_processes");
      await invokeSafe("exit_app");
    } catch (error) {
      message.error(t("quitStopFail", { error: String(error) }));
      quitLoading.value = false;
    }
  }

  async function quitWithoutStop() {
    quitLoading.value = true;
    try {
      await invokeSafe("exit_app");
    } catch (error) {
      message.error(String(error));
      quitLoading.value = false;
    }
  }

  onMounted(async () => {
    unlisten = await listenSafe("app-quit-requested", () => {
      handleQuitRequest();
    });
  });

  onBeforeUnmount(() => {
    if (typeof unlisten === "function") unlisten();
  });

  return {
    quitOpen,
    quitRunningCount,
    quitLoading,
    cancelQuit,
    stopAndQuit,
    quitWithoutStop,
  };
}
