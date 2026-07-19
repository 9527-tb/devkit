<!--
  监控面板：Host 真实采样（CPU / 内存 / 磁盘 / 进程信息 / 监听端口 / 环境）。
  端口仅来自 OS LISTEN（含子进程树），不用日志推断冒充。
  对应 DESIGN.md §8.6 / process_metrics IPC
-->
<script setup>
import { computed, onMounted, onBeforeUnmount, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { createTranslator } from "../../../i18n/index.js";
import { locale, previewMode } from "../../../stores/settings.js";

const props = defineProps({
  project: { type: Object, default: null },
  running: { type: Boolean, default: false },
  pid: { type: [Number, String, null], default: null },
  action: { type: String, default: "" },
  runtimeLabel: { type: String, default: "" },
});

const t = createTranslator(locale);

/** @type {import('vue').Ref<null | Record<string, any>>} */
const metrics = ref(null);
const error = ref("");
const loading = ref(false);
let timer;
let seq = 0;

const kind = computed(() => props.project?.kind || "");
const name = computed(() => props.project?.name || t("projectFallback"));

const title = computed(() =>
  props.running
    ? t("monitorTitleRunning", { name: name.value })
    : t("monitorTitleIdle", { name: name.value }),
);

const meta = computed(() => {
  if (!props.running) return t("monitorNotRunning");
  const pid = props.pid != null && props.pid !== "" ? props.pid : "—";
  const rt = props.runtimeLabel || kindLabel(kind.value);
  return t("monitorPidMeta", { pid, runtime: rt });
});

function kindLabel(k) {
  if (k === "maven") return "Java";
  if (k === "node") return "Node";
  if (k === "cargo") return "Cargo";
  if (k === "gradle") return "Gradle";
  return k || "—";
}

const displayAction = computed(() =>
  String(props.action || "").replace(/^script:/, "") || "—",
);

/** 仅展示 Host 采样到的真实 LISTEN 端口（含子进程树），不用日志/命令推断值冒充 */
const listenPorts = computed(() => {
  const fromMetrics = metrics.value?.ports;
  return Array.isArray(fromMetrics) ? fromMetrics.filter((p) => Number(p) > 0) : [];
});

const listenSocket = computed(() => {
  if (!listenPorts.value.length) return "—";
  return listenPorts.value.map((p) => `tcp :${p} LISTEN`).join(", ");
});

function fmtBytes(n) {
  const v = Number(n) || 0;
  if (v >= 1024 ** 3) return `${(v / 1024 ** 3).toFixed(2)} GB`;
  if (v >= 1024 ** 2) return `${(v / 1024 ** 2).toFixed(1)} MB`;
  if (v >= 1024) return `${(v / 1024).toFixed(1)} KB`;
  return `${Math.round(v)} B`;
}

function fmtRate(bps) {
  const v = Number(bps) || 0;
  if (v >= 1024 ** 2) return `${(v / 1024 ** 2).toFixed(2)} MB/s`;
  if (v >= 1024) return `${(v / 1024).toFixed(1)} KB/s`;
  return `${Math.round(v)} B/s`;
}

function fmtDuration(sec) {
  const s = Math.max(0, Math.floor(Number(sec) || 0));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const r = s % 60;
  return [h, m, r].map((x) => String(x).padStart(2, "0")).join(":");
}

function fmtStart(unixSec) {
  const t = Number(unixSec) || 0;
  if (!t) return "—";
  try {
    return new Date(t * 1000).toLocaleString();
  } catch {
    return String(t);
  }
}

const memPct = computed(() => {
  const m = metrics.value;
  if (!m?.memoryBytes || !m?.virtualMemoryBytes) return 0;
  // 虚拟内存在 macOS 上可能虚高，进度条用 RSS 相对一个合理上限展示
  const rss = Number(m.memoryBytes);
  const cap = Math.max(rss * 1.5, 512 * 1024 * 1024);
  return Math.min(100, Math.round((rss / cap) * 100));
});

const envRows = computed(() => {
  const list = metrics.value?.environ;
  if (!Array.isArray(list)) return [];
  return list.map((pair) => {
    if (Array.isArray(pair)) return [pair[0], pair[1]];
    return [pair?.[0] ?? "", pair?.[1] ?? ""];
  });
});

async function fetchMetrics() {
  const pid = Number(props.pid);
  if (!props.running || !Number.isFinite(pid) || pid <= 0) {
    metrics.value = null;
    error.value = "";
    return;
  }
  const my = ++seq;
  loading.value = true;
  try {
    const data = await invoke("process_metrics", { pid });
    if (my !== seq) return;
    metrics.value = data;
    error.value = "";
  } catch (e) {
    if (my !== seq) return;
    error.value = String(e);
    metrics.value = null;
  } finally {
    if (my === seq) loading.value = false;
  }
}

function refresh() {
  fetchMetrics();
}

watch(
  () => [props.running, props.pid],
  () => {
    fetchMetrics();
  },
  { immediate: true },
);

onMounted(() => {
  timer = window.setInterval(() => {
    if (props.running) fetchMetrics();
  }, 2000);
});
onBeforeUnmount(() => {
  if (timer) window.clearInterval(timer);
});

defineExpose({ refresh });
</script>

<template>
  <div class="monitor-pane">
    <div class="monitor-bar">
      <span>{{ title }}</span>
      <span class="monitor-meta">{{ meta }}</span>
    </div>
    <div class="monitor-body">
      <div v-if="!running" class="monitor-empty">
        {{ t("monitorIdleHint") }}
      </div>
      <div v-else-if="error && !metrics" class="monitor-empty">
        {{ error }}
      </div>
      <template v-else-if="metrics">
        <div class="mon-cards">
          <div class="mon-card">
            <div class="k">CPU</div>
            <div class="v">{{ Number(metrics.cpuPercent || 0).toFixed(1) }}%</div>
            <div class="s">{{ t("monitorCpuHint") }}</div>
            <div class="bar">
              <i :style="{ width: `${Math.min(100, Number(metrics.cpuPercent) || 0)}%` }" />
            </div>
          </div>
          <div class="mon-card">
            <div class="k">{{ t("monitorMemory") }}</div>
            <div class="v">{{ fmtBytes(metrics.memoryBytes) }}</div>
            <div class="s">{{ t("monitorMemorySub", { virtual: fmtBytes(metrics.virtualMemoryBytes) }) }}</div>
            <div class="bar"><i :style="{ width: `${memPct}%` }" /></div>
          </div>
          <div class="mon-card">
            <div class="k">{{ t("monitorNetwork") }}</div>
            <div class="v">{{ listenPorts.length ? `:${listenPorts[0]}` : t("monitorNoListen") }}</div>
            <div class="s">{{ listenSocket }}</div>
          </div>
          <div class="mon-card">
            <div class="k">{{ t("monitorDisk") }}</div>
            <div class="v">R {{ fmtRate(metrics.diskReadBps) }}</div>
            <div class="s">W {{ fmtRate(metrics.diskWriteBps) }}</div>
          </div>
        </div>

        <div class="mon-sections">
          <section class="mon-section">
            <h4>{{ t("monitorProcess") }}</h4>
            <div class="mon-kv">
              <span class="k">PID</span><span class="v">{{ metrics.pid }}</span>
              <span class="k">Parent</span>
              <span class="v">{{ metrics.parentPid ?? "—" }}</span>
              <span class="k">Name</span><span class="v">{{ metrics.name || "—" }}</span>
              <span class="k">Status</span><span class="v">{{ metrics.status || "—" }}</span>
              <span class="k">User</span><span class="v">{{ metrics.user || "—" }}</span>
              <span class="k">Action</span><span class="v">{{ displayAction }}</span>
              <span class="k">Command</span><span class="v">{{ metrics.cmd || "—" }}</span>
              <span class="k">Exe</span><span class="v">{{ metrics.exe || "—" }}</span>
              <span class="k">CWD</span><span class="v">{{ metrics.cwd || project?.path || "—" }}</span>
              <span class="k">Sockets</span><span class="v">{{ listenSocket }}</span>
            </div>
          </section>

          <section class="mon-section">
            <h4>{{ t("monitorSummary") }}</h4>
            <div class="mon-kv">
              <span class="k">CPU</span>
              <span class="v">{{ Number(metrics.cpuPercent || 0).toFixed(2) }}%</span>
              <span class="k">RSS</span>
              <span class="v">{{ fmtBytes(metrics.memoryBytes) }}</span>
              <span class="k">Virtual</span>
              <span class="v">{{ fmtBytes(metrics.virtualMemoryBytes) }}</span>
              <span class="k">Disk Read</span>
              <span class="v">{{ fmtRate(metrics.diskReadBps) }}</span>
              <span class="k">Disk Write</span>
              <span class="v">{{ fmtRate(metrics.diskWriteBps) }}</span>
              <span class="k">Started</span>
              <span class="v">{{ fmtStart(metrics.startTime) }}</span>
              <span class="k">Uptime</span>
              <span class="v">{{ fmtDuration(metrics.runTime) }}</span>
              <span class="k">{{ t("monitorSample") }}</span>
              <span class="v">{{ loading ? t("refreshing") : t("monitorSampleRate") }}</span>
            </div>
          </section>

          <section class="mon-section full">
            <h4>{{ t("monitorEnv") }}</h4>
            <div v-if="!envRows.length" class="mon-env-empty">{{ t("monitorEnvEmpty") }}</div>
            <div v-else class="mon-env">
              <div v-for="[ek, ev] in envRows" :key="ek" class="mon-env-row">
                <span class="k" :title="ek">{{ ek }}</span>
                <span class="v" :title="ev">{{ ev }}</span>
              </div>
            </div>
          </section>
        </div>
      </template>
      <div v-else class="monitor-empty">{{ t("monitorCollecting") }}</div>
    </div>
  </div>
</template>

<style scoped>
.monitor-pane {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--line, #d7e0dc);
  border-radius: var(--radius);
  background: var(--panel, #fff);
  overflow: hidden;
}
.monitor-bar {
  flex: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-bottom: 1px solid var(--line, #d7e0dc);
  background: var(--input-bg, #f4f8f6);
  font-size: 12px;
  font-weight: 600;
}
.monitor-meta {
  color: var(--muted, #6b7a76);
  font-size: 11px;
  font-weight: 600;
}
.monitor-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.monitor-empty {
  flex: 1;
  display: grid;
  place-items: center;
  color: var(--muted, #6b7a76);
  font-size: 13px;
  font-weight: 500;
  min-height: 160px;
  padding: 16px;
  text-align: center;
}
.mon-cards {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
}
@media (max-width: 980px) {
  .mon-cards {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
.mon-card {
  padding: 10px 12px;
  border-radius: var(--radius);
  border: 1px solid var(--line, #d7e0dc);
  background: var(--surface-muted, #f5f7f6);
}
.mon-card .k {
  color: var(--muted, #6b7a76);
  font-size: 11px;
  font-weight: 600;
}
.mon-card .v {
  margin-top: 4px;
  font-size: 18px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--ink, #15201e);
  font-variant-numeric: tabular-nums;
}
.mon-card .s {
  margin-top: 2px;
  color: var(--muted, #6b7a76);
  font-size: 11px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.mon-card .bar {
  margin-top: 8px;
  height: 4px;
  border-radius: var(--radius);
  background: var(--chip-bg, #dce8e3);
  overflow: hidden;
}
.mon-card .bar > i {
  display: block;
  height: 100%;
  border-radius: var(--radius);
  background: linear-gradient(90deg, var(--teal-mid, #1a8f84), var(--teal, #0f766e));
}
.mon-sections {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
@media (max-width: 980px) {
  .mon-sections {
    grid-template-columns: 1fr;
  }
}
.mon-section {
  border: 1px solid var(--line, #d7e0dc);
  border-radius: var(--radius);
  background: #fff;
  overflow: hidden;
  min-width: 0;
}
.mon-section.full {
  grid-column: 1 / -1;
}
.mon-section h4 {
  margin: 0;
  padding: 8px 12px;
  border-bottom: 1px solid var(--line-soft, #eef3f0);
  background: var(--panel, #fff);
  font-size: 12px;
  font-weight: 700;
  color: var(--ink-soft, #30433e);
}
.mon-kv {
  display: grid;
  grid-template-columns: 120px 1fr;
  gap: 4px 10px;
  padding: 10px 12px;
  font-size: 12px;
}
.mon-kv .k {
  color: var(--muted, #6b7a76);
  font-weight: 600;
}
.mon-kv .v {
  color: var(--ink-strong, #24322e);
  font-family: "IBM Plex Mono", ui-monospace, monospace;
  font-size: 11.5px;
  word-break: break-all;
}
.mon-env {
  max-height: 220px;
  overflow: auto;
  padding: 6px 0;
}
.mon-env-empty {
  padding: 16px 12px;
  color: var(--muted, #6b7a76);
  font-size: 12px;
}
.mon-env-row {
  display: grid;
  grid-template-columns: minmax(100px, 160px) 1fr;
  gap: 8px;
  padding: 4px 12px;
  font-size: 11.5px;
}
.mon-env-row:nth-child(odd) {
  background: var(--surface-muted, #f5f7f6);
}
.mon-env-row .k {
  color: var(--muted, #6b7a76);
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.mon-env-row .v {
  color: var(--muted, #6b7a76);
  font-family: "IBM Plex Mono", ui-monospace, monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
