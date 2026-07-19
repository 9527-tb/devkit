<!--
  项目头：左侧名称 + 多实例选择；右侧全部操作按钮（同一 a-space）。
-->
<script setup>
import { computed } from "vue";
import { CodeOutlined, CodeSandboxOutlined } from "@antdv-next/icons";
import { actionButtonCount } from "../../stores/settings.js";

const props = defineProps({
  project: { type: Object, required: true },
  t: { type: Function, required: true },
  showInstanceSelect: { type: Boolean, default: false },
  selectedPid: { type: [Number, String, null], default: null },
  instanceOptions: { type: Array, default: () => [] },
  actions: { type: Array, default: () => [] },
  displayAction: { type: Function, required: true },
  hasRunning: { type: Boolean, default: false },
  stopLabel: { type: String, default: "" },
});

const emit = defineEmits([
  "open-editor",
  "open-terminal",
  "update:selectedPid",
  "run",
  "stop",
]);

const resolvedStopLabel = computed(() => props.stopLabel || props.t("stop"));
const primaryActions = computed(() => props.actions.slice(0, actionButtonCount.value));
const overflowActions = computed(() => props.actions.slice(actionButtonCount.value));

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
  <div class="proj-bar">
    <div class="proj-left">
      <strong class="proj-name" :title="project.path">{{ project.name }}</strong>
      <a-select
        v-if="showInstanceSelect"
        size="small"
        :value="selectedPid"
        class="instance-select"
        :options="instanceOptions"
        :placeholder="t('selectInstance')"
        @update:value="emit('update:selectedPid', $event)"
      />
    </div>

    <a-space :size="2" align="center" class="proj-actions">
      <a-tooltip :title="t('openInEditor')" :mouse-enter-delay="0">
        <a-button
          type="text"
          size="small"
          class="proj-ext-btn"
          @click="emit('open-editor')"
        >
          <template #icon><CodeSandboxOutlined /></template>
        </a-button>
      </a-tooltip>
      <a-tooltip :title="t('openInTerminal')" :mouse-enter-delay="0">
        <a-button
          type="text"
          size="small"
          class="proj-ext-btn"
          @click="emit('open-terminal')"
        >
          <template #icon><CodeOutlined /></template>
        </a-button>
      </a-tooltip>
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
      <a-button size="small" danger :disabled="!hasRunning" @click="emit('stop')">
        {{ resolvedStopLabel }}
      </a-button>
    </a-space>
  </div>
</template>

<style scoped>
.proj-bar {
  flex: none;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px;
  box-sizing: border-box;
  overflow: hidden;
  border-bottom: 1px solid var(--line, #d7e0dc);
  background: color-mix(in srgb, var(--panel, #fff) 90%, transparent);
}
.proj-left {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}
.proj-name {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: -0.02em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.instance-select {
  width: 120px;
  flex: none;
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
.proj-actions {
  margin-left: auto;
  min-width: 0;
}
.proj-ext-btn {
  width: 24px !important;
  height: 24px !important;
  min-width: 24px !important;
  padding: 0 !important;
  display: inline-flex !important;
  align-items: center;
  justify-content: center;
  color: var(--ink-soft, #30433e);
}
.proj-ext-btn:hover {
  color: var(--teal, #0f766e);
}
.action-more-btn {
  width: 24px;
  min-width: 24px;
  padding-inline: 0;
  letter-spacing: 0.08em;
}
</style>
