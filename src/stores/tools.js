/**
 * 工具页导航状态（与 settingsPage 互斥的全屏页）。
 * 当前工具 id 见 features/tools/registry.js。
 */

import { ref } from "vue";
import { defaultToolId } from "../features/tools/registry.js";

/** 是否显示工具页 */
export const toolsPage = ref(false);

/** 当前工具 id（与 registry / 后端 tools::registry 对齐） */
export const toolsCat = ref(defaultToolId());
