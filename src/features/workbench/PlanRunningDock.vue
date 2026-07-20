<!--
  编排计划执行中：底部停止条。
-->
<script setup>
import { computed } from "vue";

const props = defineProps({
  t: { type: Function, required: true },
  running: { type: Object, default: null },
});

const emit = defineEmits(["stop"]);

const visible = computed(() => !!(props.running && props.running.id));
const name = computed(() => props.running?.name || "");
const count = computed(() => props.running?.projects?.length || 0);
</script>

<template>
  <div v-if="visible" class="plan-dock">
    <span class="plan-dock-label">
      {{ t("runPlanRunning", { name, n: count }) }}
    </span>
    <span class="plan-dock-grow" />
    <a-button size="small" danger type="primary" @click="emit('stop')">
      {{ t("runPlanStop") }}
    </a-button>
  </div>
</template>

<style scoped>
.plan-dock {
  position: absolute;
  left: 8px;
  right: 8px;
  bottom: 8px;
  z-index: 41;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  border: 1px solid color-mix(in srgb, #ef4444 35%, var(--line));
  border-radius: var(--radius, 3px);
  background: color-mix(in srgb, var(--panel) 92%, #fee2e2);
  box-shadow: 0 8px 20px rgba(16, 28, 24, 0.12);
}
.plan-dock-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-soft);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.plan-dock-grow {
  flex: 1;
}
</style>
