<script setup>
import { computed } from "vue";
import { Handle, Position } from "@vue-flow/core";

const props = defineProps({
  data: { type: Object, default: () => ({}) },
  selected: { type: Boolean, default: false },
});

const title = computed(() => {
  const act = String(props.data.action || "").replace(/^script:/, "") || "…";
  const rel = String(props.data.relPath || "?");
  const name = rel.split("/").pop() || rel;
  return `${name} · ${act}`;
});

const waitLabel = computed(() =>
  props.data.wait === "complete"
    ? props.data.waitCompleteLabel || "wait"
    : props.data.waitSpawnLabel || "spawn",
);
</script>

<template>
  <div class="pn pn-step" :class="{ on: selected }" @dblclick.stop="data.onEdit?.()">
    <Handle id="in" type="target" :position="Position.Left" class="pn-handle" />
    <div class="pn-body">
      <span class="pn-kind">{{ data.kind || "—" }}</span>
      <span class="pn-title">{{ title }}</span>
      <span class="pn-wait">{{ waitLabel }}</span>
    </div>
    <Handle id="out" type="source" :position="Position.Right" class="pn-handle" />
  </div>
</template>

<style scoped>
.pn {
  width: 176px;
  border: 1px solid var(--line);
  border-radius: var(--radius, 3px);
  background: var(--panel, #fff);
  box-shadow: 0 1px 2px rgba(16, 28, 24, 0.06);
}
.pn.on {
  border-color: var(--teal);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--teal) 25%, transparent);
}
.pn-body {
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.pn-kind {
  font-size: 10px;
  font-weight: 700;
  color: var(--muted);
  text-transform: uppercase;
}
.pn-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-strong);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.pn-wait {
  font-size: 10px;
  color: var(--muted);
}
.pn-handle {
  width: 10px !important;
  height: 10px !important;
  background: var(--teal) !important;
  border: 2px solid #fff !important;
}
</style>
