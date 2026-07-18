<!--
  日志面板：流式输出、换行、清空、链接点击。
  对应 DESIGN.md §8 panel.logs
-->
<script setup>
import { nextTick, ref, watch } from "vue";
import { formatLogHtml, formatLogLine } from "../../../shared/logFormat.js";
import { logWrap } from "../../../stores/settings.js";
import { openUrl } from "@tauri-apps/plugin-opener";
// 自动换行 checkbox 在面板 Tab 行（与原型 logTools 一致）

// DONE(fe-panel-logs): 增量渲染减轻启动卡顿 — DESIGN §8

const props = defineProps({
  t: { type: Function, required: true },
  projectName: { type: String, default: "" },
  lines: { type: Array, default: () => [] },
});

const emit = defineEmits(["clear"]);

const logEl = ref(null);
const logsPinned = ref(false);
const logHtml = ref("");
let renderedLen = 0;

function onLogScroll() {
  const el = logEl.value;
  if (!el) return;
  const gap = el.scrollHeight - el.scrollTop - el.clientHeight;
  logsPinned.value = gap > 40;
}

async function scrollLogsToBottom() {
  await nextTick();
  const el = logEl.value;
  if (el && !logsPinned.value) el.scrollTop = el.scrollHeight;
}

function rebuildHtml(lines) {
  logHtml.value = formatLogHtml(lines);
  renderedLen = lines.length;
}

function appendHtml(lines) {
  const start = renderedLen;
  const chunk = [];
  for (let i = start; i < lines.length; i += 1) {
    chunk.push(formatLogLine(lines[i]));
  }
  if (!chunk.length) return;
  logHtml.value = logHtml.value
    ? `${logHtml.value}\n${chunk.join("\n")}`
    : chunk.join("\n");
  renderedLen = lines.length;
}

function canAppend(prev, next) {
  if (!prev?.length || !next || next.length <= prev.length) return false;
  if (prev.length !== renderedLen) return false;
  // 确认 next 是 prev 的前缀扩展，避免切换实例时误追加
  const sample = Math.min(prev.length, 8);
  for (let i = 0; i < sample; i += 1) {
    if (prev[i] !== next[i]) return false;
  }
  if (prev[prev.length - 1] !== next[prev.length - 1]) return false;
  return true;
}

watch(
  () => props.lines,
  (lines, prev) => {
    const list = lines || [];
    if (canAppend(prev, list)) {
      // 仅追加：避免启动期整表重格式化导致卡顿
      appendHtml(list);
    } else {
      rebuildHtml(list);
      logsPinned.value = false;
    }
    if (!logsPinned.value) scrollLogsToBottom();
  },
  { deep: false },
);

/** 普通点击不跳转；Ctrl/⌘ + 点击才用系统浏览器打开 */
async function onLogClick(event) {
  const a = event.target?.closest?.("a.log-link, a[href]");
  if (!a) return;
  // 始终拦截默认跳转，避免 WebView 直接导航离开应用
  event.preventDefault();
  event.stopPropagation();
  if (!(event.ctrlKey || event.metaKey)) return;
  const href = a.getAttribute("href") || a.href;
  if (!href || href.startsWith("javascript:")) return;
  try {
    await openUrl(href);
  } catch {
    window.open(href, "_blank", "noopener");
  }
}

/** 拦截中键等非主键打开链接 */
function onLogAuxClick(event) {
  if (event.target?.closest?.("a.log-link, a[href]")) {
    event.preventDefault();
    event.stopPropagation();
  }
}
</script>

<template>
  <div class="console">
    <div class="console-bar">
      <span>{{ t("logsOutput", { name: projectName }) }}</span>
      <div class="console-bar-actions">
        <span class="console-hint">{{ t("logsOpenLinkHint") }}</span>
        <a-button type="link" size="small" class="clear-btn" @click="emit('clear')">
          {{ t("clear") }}
        </a-button>
      </div>
    </div>
    <div ref="logEl" class="console-body" @scroll="onLogScroll">
      <pre
        v-if="lines.length"
        class="console-log"
        :class="{ wrap: logWrap }"
        v-html="logHtml"
        @click="onLogClick"
        @auxclick="onLogAuxClick"
      ></pre>
      <div v-else class="muted">{{ t("logsEmpty") }}</div>
    </div>
  </div>
</template>

<style scoped>
.clear-btn.ant-btn {
  height: auto;
  padding: 0 4px;
  font-size: 12px;
  line-height: 1.2;
}
</style>
