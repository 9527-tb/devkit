<script setup>
import { Handle, Position } from "@vue-flow/core";

defineProps({
  data: { type: Object, default: () => ({}) },
  selected: { type: Boolean, default: false },
});
</script>

<template>
  <div class="pn pn-start" :class="{ on: selected }">
    <span>{{ data.label || "Start" }}</span>
    <Handle id="out" type="source" :position="Position.Right" class="pn-handle" />
  </div>
</template>

<style scoped>
.pn {
  min-width: 96px;
  padding: 10px 16px;
  border: 1px solid var(--line);
  border-radius: var(--radius, 3px);
  background: var(--surface-tint, #e8efec);
  font-size: 12px;
  font-weight: 800;
  text-align: center;
  color: var(--ink-strong);
  box-shadow: 0 1px 2px rgba(16, 28, 24, 0.06);
}
.pn.on {
  border-color: var(--teal);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--teal) 25%, transparent);
}
.pn-handle {
  width: 10px !important;
  height: 10px !important;
  background: var(--teal) !important;
  border: 2px solid #fff !important;
}
</style>
