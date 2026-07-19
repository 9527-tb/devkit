<!--
  日志面板：流式输出、搜索、底部运行摘要；滚动贴底（上翻后暂停）。
-->
<script setup>
import { computed, nextTick, ref, watch } from "vue";
import { formatLogHtml, formatLogLine } from "../../../shared/logFormat.js";
import { logWrap } from "../../../stores/settings.js";
import { openUrl } from "@tauri-apps/plugin-opener";
import { message } from "antdv-next";

const props = defineProps({
  t: { type: Function, required: true },
  projectName: { type: String, default: "" },
  lines: { type: Array, default: () => [] },
  /** 最近一次运行摘要（RunSummary） */
  runSummary: { type: Object, default: null },
});

const emit = defineEmits(["clear"]);

const logEl = ref(null);
const logsPinned = ref(false);
const logHtml = ref("");
const search = ref("");
/** 用户手动关闭摘要条；新摘要到达时重置 */
const summaryDismissed = ref(false);
let renderedLen = 0;
let renderedKey = "";

const filteredLines = computed(() => {
  const needle = search.value.trim().toLowerCase();
  if (!needle) return props.lines || [];
  return (props.lines || []).filter((line) =>
    String(line || "").toLowerCase().includes(needle),
  );
});

const summaryVisible = computed(() => {
  if (summaryDismissed.value) return false;
  const s = props.runSummary;
  return s && (s.success === false || (Array.isArray(s.errorLines) && s.errorLines.length));
});

watch(
  () => props.runSummary,
  () => {
    summaryDismissed.value = false;
  },
);

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

function filterKey() {
  return `${search.value}|${logWrap.value}`;
}

function rebuildHtml(lines) {
  logHtml.value = formatLogHtml(lines);
  renderedLen = lines.length;
  renderedKey = filterKey();
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
  if (renderedKey !== filterKey()) return false;
  if (!prev?.length || !next || next.length <= prev.length) return false;
  if (prev.length !== renderedLen) return false;
  const sample = Math.min(prev.length, 8);
  for (let i = 0; i < sample; i += 1) {
    if (prev[i] !== next[i]) return false;
  }
  if (prev[prev.length - 1] !== next[prev.length - 1]) return false;
  return true;
}

watch(
  filteredLines,
  (lines, prev) => {
    const list = lines || [];
    if (canAppend(prev, list)) {
      appendHtml(list);
    } else {
      rebuildHtml(list);
      logsPinned.value = false;
    }
    if (!logsPinned.value) scrollLogsToBottom();
  },
  { deep: false },
);

async function onLogClick(event) {
  const a = event.target?.closest?.("a.log-link, a[href]");
  if (!a) return;
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

function onLogAuxClick(event) {
  if (event.target?.closest?.("a.log-link, a[href]")) {
    event.preventDefault();
    event.stopPropagation();
  }
}

async function copySummary() {
  const s = props.runSummary;
  if (!s) return;
  const parts = [
    `Project: ${props.projectName || s.path || ""}`,
    `Action: ${s.action || ""}`,
    `Exit: ${s.exitCode ?? "—"}`,
    `Duration: ${s.durationMs ?? 0}ms`,
    "",
    ...(s.errorLines || []),
    "",
    ...((s.hints || []).map((h) => `Hint: ${h.message}`)),
  ];
  try {
    await navigator.clipboard.writeText(parts.filter(Boolean).join("\n"));
    message.success(props.t("runSummaryCopied"));
  } catch (e) {
    message.error(String(e));
  }
}

function dismissSummary() {
  summaryDismissed.value = true;
}
</script>

<template>
  <div class="console">
    <div class="console-bar">
      <span>{{ t("logsOutput", { name: projectName }) }}</span>
      <div class="console-bar-actions">
        <input
          v-model="search"
          type="search"
          class="log-search"
          :placeholder="t('logSearch')"
          spellcheck="false"
          autocomplete="off"
        />
        <button type="button" class="console-text-btn" @click="emit('clear')">
          {{ t("clear") }}
        </button>
      </div>
    </div>

    <div ref="logEl" class="console-body" @scroll="onLogScroll">
      <pre
        v-if="filteredLines.length"
        class="console-log"
        :class="{ wrap: logWrap }"
        v-html="logHtml"
        @click="onLogClick"
        @auxclick="onLogAuxClick"
      ></pre>
      <div v-else class="muted">{{ t("logsEmpty") }}</div>
    </div>

    <div
      v-if="summaryVisible"
      class="run-summary"
      :class="runSummary.success ? 'ok' : 'fail'"
    >
      <div class="run-summary-main">
        <strong>
          {{
            runSummary.success
              ? t("runSummaryOk")
              : t("runSummaryFail", { code: runSummary.exitCode ?? "?" })
          }}
        </strong>
        <span class="run-summary-meta">
          {{ runSummary.action }} · {{ Math.round((runSummary.durationMs || 0) / 1000) }}s
        </span>
        <div class="run-summary-actions">
          <button type="button" class="console-text-btn run-summary-copy" @click="copySummary">
            {{ t("runSummaryCopy") }}
          </button>
          <button type="button" class="console-text-btn run-summary-close" @click="dismissSummary">
            {{ t("close") }}
          </button>
        </div>
      </div>
      <ul v-if="runSummary.errorLines?.length" class="run-summary-errors">
        <li v-for="(line, i) in runSummary.errorLines.slice(0, 3)" :key="i">{{ line }}</li>
      </ul>
      <ul v-if="runSummary.hints?.length" class="run-summary-hints">
        <li v-for="h in runSummary.hints" :key="h.id">{{ h.message }}</li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.log-search {
  width: 132px;
  height: 22px;
  margin: 0;
  padding: 0 8px;
  border: 1px solid #2a3834;
  border-radius: var(--radius);
  background: #111a18;
  color: #d5e3dd;
  font: 12px/22px ui-monospace, SFMono-Regular, Menlo, monospace;
  outline: none;
  -webkit-appearance: none;
  appearance: none;
}
.log-search::placeholder {
  color: #7f908a;
}
.log-search:focus {
  border-color: #3d524c;
}
.log-search::-webkit-search-cancel-button {
  -webkit-appearance: none;
  appearance: none;
}

.console-text-btn {
  border: 0;
  margin: 0;
  padding: 0;
  background: none;
  color: #7dd3c0;
  font: inherit;
  font-size: 12px;
  line-height: 1.2;
  cursor: pointer;
}
.console-text-btn:hover {
  color: #99f6e4;
}

.run-summary {
  flex: none;
  padding: 8px 10px;
  border-top: 1px solid #243330;
  font: 12px/1.45 ui-monospace, SFMono-Regular, Menlo, monospace;
  animation: run-summary-in 0.18s ease-out;
}
.run-summary.fail {
  background: #2a1616;
  color: #fecaca;
}
.run-summary.ok {
  background: #14241c;
  color: #bbf7d0;
}
.run-summary-main {
  display: flex;
  gap: 10px;
  align-items: baseline;
  flex-wrap: wrap;
}
.run-summary-meta {
  opacity: 0.75;
}
.run-summary-actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 12px;
}
.run-summary-copy,
.run-summary-close {
  flex: none;
}
.run-summary.fail .run-summary-copy,
.run-summary.fail .run-summary-close {
  color: #fca5a5;
}
.run-summary.ok .run-summary-copy,
.run-summary.ok .run-summary-close {
  color: #86efac;
}
.run-summary-errors,
.run-summary-hints {
  margin: 6px 0 0;
  padding-left: 16px;
}
.run-summary-hints {
  color: #fde68a;
}

@keyframes run-summary-in {
  from {
    opacity: 0;
    transform: translateY(6px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
