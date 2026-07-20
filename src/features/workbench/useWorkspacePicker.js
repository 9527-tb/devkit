/**
 * 工作区选择：切换单根 / 追加根 / 打开历史。
 */

import { open } from "@tauri-apps/plugin-dialog";
import { message } from "antdv-next";
import { previewMode, settings } from "../../stores/settings.js";
import {
  setWorkspaceRoot,
  addWorkspaceRoot,
} from "../../stores/workspace.js";

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

/** 追加挂载目录（不替换现有根） */
export async function addWorkspaceFolder(t) {
  if (previewMode.value) {
    message.info(t("previewNeedTauriShort"));
    return false;
  }
  const picked = await open({
    directory: true,
    multiple: false,
    title: t("addWorkspaceRoot"),
  });
  if (!picked) return false;
  const max = settings.value.general?.maxWorkspaceRoots ?? 10;
  const result = addWorkspaceRoot(picked, max);
  if (!result.ok) {
    if (result.reason === "limit") {
      message.warning(t("workspaceRootLimit", { n: max }));
    } else if (result.reason === "exists") {
      message.warning(t("workspaceRootExists"));
    }
    return false;
  }
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
