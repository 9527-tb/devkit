<!--
  工作区体检 Popover：列表 + 去处理。标题栏用 icon 触发器。
-->
<script setup>
import { computed } from "vue";
import {
  WarningOutlined,
  CheckCircleOutlined,
  MedicineBoxOutlined,
} from "@antdv-next/icons";

const props = defineProps({
  t: { type: Function, required: true },
  report: { type: Object, default: null },
  loading: { type: Boolean, default: false },
  /** icon：标题栏图标按钮；tag：旧标签样式 */
  trigger: { type: String, default: "icon" },
});

const emit = defineEmits(["handle", "refresh"]);

const items = computed(() => props.report?.items || []);
const ok = computed(() => props.report?.ok !== false);
const warnCount = computed(
  () => items.value.filter((i) => i.level === "warn" || i.level === "error").length,
);

const tagColor = computed(() => {
  if (props.loading) return "processing";
  if (!props.report) return "default";
  if (warnCount.value) return "warning";
  return ok.value ? "success" : "error";
});

const tagLabel = computed(() => {
  if (props.loading) return props.t("healthChecking");
  if (!props.report) return props.t("healthUnknown");
  if (warnCount.value) return props.t("healthWarnCount", { n: warnCount.value });
  return props.t("healthOk");
});

const iconTitle = computed(() => {
  if (props.loading) return props.t("healthChecking");
  if (!props.report) return props.t("healthTitle");
  if (warnCount.value) return props.t("healthWarnCount", { n: warnCount.value });
  return props.t("healthOk");
});

const iconTone = computed(() => {
  if (props.loading) return "loading";
  if (!props.report) return "muted";
  if (warnCount.value) return "warn";
  return ok.value ? "ok" : "warn";
});

function levelClass(level) {
  if (level === "error") return "level-error";
  if (level === "warn") return "level-warn";
  return "level-ok";
}

function onHandle(item) {
  emit("handle", { action: item.action, port: item.port });
}
</script>

<template>
  <a-popover
    trigger="click"
    placement="bottomRight"
    :arrow="false"
    :styles="{ container: { padding: '8px' } }"
    :classes="{ root: 'health-popover' }"
  >
    <template #content>
      <div class="health-panel">
        <div class="health-head">
          <strong>{{ t("healthTitle") }}</strong>
          <button type="button" class="health-refresh" :disabled="loading" @click="emit('refresh')">
            {{ loading ? t("refreshing") : t("refresh") }}
          </button>
        </div>
        <div v-if="loading && !items.length" class="health-empty">{{ t("healthChecking") }}</div>
        <ul v-else-if="items.length" class="health-list">
          <li v-for="item in items" :key="item.id" :class="levelClass(item.level)">
            <span class="health-msg">{{ item.message }}</span>
            <a-button
              v-if="item.action"
              type="link"
              size="small"
              class="health-action"
              @click="onHandle(item)"
            >
              {{ t("healthGoFix") }}
            </a-button>
          </li>
        </ul>
        <div v-else class="health-empty">{{ t("healthNoData") }}</div>
      </div>
    </template>

    <a-button
      v-if="trigger === 'icon'"
      type="text"
      class="health-icon-btn"
      :class="`is-${iconTone}`"
      :title="iconTitle"
      :loading="loading"
    >
      <template #icon>
        <WarningOutlined v-if="warnCount" />
        <CheckCircleOutlined v-else-if="report && !loading" />
        <MedicineBoxOutlined v-else />
      </template>
    </a-button>
    <a-tag v-else :color="tagColor" class="health-tag">
      <CheckCircleOutlined v-if="!warnCount && report && !loading" />
      <WarningOutlined v-else-if="warnCount" />
      {{ tagLabel }}
    </a-tag>
  </a-popover>
</template>

<style scoped>
.health-icon-btn {
  width: 28px !important;
  height: 28px !important;
  min-width: 28px !important;
  padding: 0 !important;
  display: inline-flex !important;
  align-items: center;
  justify-content: center;
  color: var(--ink-soft);
  -webkit-app-region: no-drag;
  app-region: no-drag;
}
.health-icon-btn.is-ok {
  color: #059669;
}
.health-icon-btn.is-warn {
  color: #d97706;
}
.health-icon-btn.is-muted {
  color: var(--ink-soft);
}
.health-tag {
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  font-weight: 600;
}
.health-panel {
  width: 300px;
}
.health-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}
.health-head strong {
  font-size: 12px;
}
.health-refresh {
  border: 0;
  background: none;
  padding: 0;
  color: var(--teal, #0f766e);
  font: inherit;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
}
.health-refresh:disabled {
  opacity: 0.5;
  cursor: default;
}
.health-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 280px;
  overflow: auto;
}
.health-list li {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
  padding: 6px 8px;
  border-radius: var(--radius);
  font-size: 11px;
  line-height: 1.4;
}
.level-ok {
  background: color-mix(in srgb, #059669 12%, transparent);
}
.level-warn {
  background: color-mix(in srgb, #d97706 14%, transparent);
}
.level-error {
  background: color-mix(in srgb, #dc2626 12%, transparent);
}
.health-msg {
  flex: 1;
  min-width: 0;
}
.health-action {
  flex: none;
  padding: 0;
  height: auto;
  font-size: 11px;
}
.health-empty {
  color: var(--muted, #6b7a76);
  font-size: 11px;
  padding: 8px 0;
}
</style>
