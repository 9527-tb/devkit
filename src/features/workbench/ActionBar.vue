<!--
  Action 栏：实例选择 | 前 N 按钮 + ··· | 分隔 | 停止。对齐原型 .proj-actions
  主色按钮使用 a-button，跟随 ConfigProvider colorPrimary。
  对应 DESIGN.md §10 / 原型 ActionBar
-->
<script setup>
import { computed } from "vue";
import { createTranslator } from "../../i18n/index.js";
import { actionButtonCount, locale } from "../../stores/settings.js";

// DONE(ux-action-overflow): 前 N 个按钮 + ⋯；N 来自设置 actionButtonCount — DESIGN §10

const t = createTranslator(locale);

const props = defineProps({
  actions: { type: Array, default: () => [] },
  displayAction: { type: Function, required: true },
  hasRunning: { type: Boolean, default: false },
  showInstanceSelect: { type: Boolean, default: false },
  selectedPid: { type: [Number, String, null], default: null },
  instanceOptions: { type: Array, default: () => [] },
  stopLabel: { type: String, default: "" },
});

const resolvedStopLabel = computed(() => props.stopLabel || t("stop"));

const emit = defineEmits(["run", "stop", "update:selectedPid"]);

const visibleCount = computed(() => actionButtonCount.value);
const primaryActions = computed(() => props.actions.slice(0, visibleCount.value));
const overflowActions = computed(() => props.actions.slice(visibleCount.value));

const overflowMenu = computed(() => ({
  items: overflowActions.value.map((action) => ({
    key: action,
    label: props.displayAction(action),
  })),
  onClick: ({ key }) => emit("run", key),
}));

function isPrimary(action) {
  return ["dev", "start", "spring-boot:run", "run"].includes(props.displayAction(action));
}
</script>

<template>
  <div class="action-bar">
    <a-space :size="8" align="center" class="action-main">
      <a-select
        v-if="showInstanceSelect"
        size="small"
        :value="selectedPid"
        class="instance-select"
        :options="instanceOptions"
        :placeholder="t('selectInstance')"
        @update:value="emit('update:selectedPid', $event)"
      />
      <a-space :size="6" align="center">
        <a-button
          v-for="action in primaryActions"
          :key="action"
          size="small"
          :type="isPrimary(action) ? 'primary' : 'default'"
          @click="emit('run', action)"
        >
          {{ displayAction(action) }}
        </a-button>
        <a-dropdown v-if="overflowActions.length" :menu="overflowMenu" :trigger="['click']">
          <a-button size="small" class="action-more-btn" :title="t('moreActions')">
            ···
          </a-button>
        </a-dropdown>
      </a-space>
    </a-space>

    <div class="action-divider" aria-hidden="true" />

    <a-button size="small" danger :disabled="!hasRunning" @click="emit('stop')">
      {{ resolvedStopLabel }}
    </a-button>
  </div>
</template>

<style scoped>
.action-bar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  min-width: 0;
  height: 24px;
  min-height: 24px;
  max-height: 24px;
}
.action-main {
  min-width: 0;
}
.action-divider {
  flex: none;
  width: 1px;
  height: 14px;
  background: var(--chip-bg, #dce8e3);
}
.instance-select {
  width: 120px;
}
.instance-select :deep(.ant-select-selector) {
  height: 24px !important;
  min-height: 24px !important;
  align-items: center;
}
.instance-select :deep(.ant-select-selection-item),
.instance-select :deep(.ant-select-selection-placeholder) {
  line-height: 22px !important;
}
.action-more-btn {
  width: 24px;
  min-width: 24px;
  padding-inline: 0;
  letter-spacing: 0.08em;
}
</style>
