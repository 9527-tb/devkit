<!--
  项目头：名称/路径、runtime pill、动作区槽位。对齐原型 .proj-bar
  对应 DESIGN.md §10 / 原型 ProjectHeader
-->
<script setup>
import { computed } from "vue";

// DONE(ux-project-header): 项目头 / runtime pill — DESIGN §10

const props = defineProps({
  project: { type: Object, required: true },
  runtimeLabel: { type: String, default: "" },
});

const pathLine = computed(() => {
  const p = props.project;
  if (!p) return "";
  const parts = String(p.path || "").replace(/\\/g, "/").split("/").filter(Boolean);
  const short = parts.length <= 2 ? p.path : `…/${parts.slice(-2).join("/")}`;
  const kind =
    p.kind === "maven"
      ? "Maven"
      : p.kind === "node"
        ? "Node"
        : p.kind === "cargo"
          ? "Cargo"
          : p.kind === "gradle"
            ? "Gradle"
            : p.kind;
  return `${short} · ${kind}`;
});
</script>

<template>
  <div class="proj-bar">
    <div class="proj-title">
      <strong>{{ project.name }}</strong>
      <span :title="project.path">{{ pathLine }}</span>
    </div>
    <span v-if="runtimeLabel" class="runtime-pill">
      <i />
      {{ runtimeLabel }}
    </span>
    <div class="proj-actions">
      <slot name="actions" />
    </div>
  </div>
</template>

<style scoped>
.proj-bar {
  flex: none;
  display: flex;
  align-items: center;
  gap: 10px;
  height: 52px;
  min-height: 52px;
  max-height: 52px;
  padding: 0 8px;
  box-sizing: border-box;
  overflow: hidden;
  border-bottom: 1px solid var(--line, #d7e0dc);
  background: color-mix(in srgb, var(--panel, #fff) 90%, transparent);
}
.proj-title {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.proj-title strong {
  font-size: 13px;
  font-weight: 700;
  letter-spacing: -0.02em;
}
.proj-title span {
  color: var(--muted, #6b7a76);
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.runtime-pill {
  flex: none;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  height: 24px;
  padding: 0 8px;
  border-radius: var(--radius);
  background: var(--surface-hover, #f3f7f5);
  color: var(--ink-soft, #30433e);
  font-size: 11px;
  font-weight: 600;
  border: 1px solid var(--line, #d7e0dc);
}
.runtime-pill i {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--teal-mid, #1a8f84);
}
.proj-actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}
</style>
