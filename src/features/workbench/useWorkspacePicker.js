/**
 * 工作区选择：选目录 / 打开历史。可在标题栏与工作台共用。
 */

import { open } from "@tauri-apps/plugin-dialog";
import { message } from "antdv-next";
import { previewMode } from "../../stores/settings.js";
import { setWorkspaceRoot } from "../../stores/workspace.js";

export async function chooseNewWorkspace(t) {
  if (previewMode.value) {
    message.info(t("previewNeedTauriShort"));
    return false;
  }
  const picked = await open({
    directory: true,
    multiple: false,
    title: t("chooseNewProject"),
  });
  if (!picked) return false;
  setWorkspaceRoot(picked);
  return true;
}

export function openHistoryWorkspace(path, t) {
  if (!path) return false;
  if (previewMode.value) {
    message.info(t("previewNeedTauriShort"));
    return false;
  }
  setWorkspaceRoot(path);
  return true;
}
