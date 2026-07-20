<!--
  标题栏 Git 状态：pill（原型）或 icon。
-->
<script setup>
import { computed } from "vue";
import { BranchesOutlined } from "@antdv-next/icons";

const props = defineProps({
  t: { type: Function, required: true },
  status: { type: Object, default: null },
  /** pill | icon */
  variant: { type: String, default: "pill" },
});

const visible = computed(
  () => !!(props.status?.available && props.status.branch),
);

const title = computed(() => {
  if (!visible.value) return props.t("gitUnavailable");
  const parts = [props.status.branch];
  if (props.status.dirty) parts.push(props.t("gitDirty"));
  return parts.join(" · ");
});

const pillLabel = computed(() => {
  if (!visible.value) return props.t("gitUnavailable");
  return title.value;
});
</script>

<template>
  <a-popover
    v-if="visible"
    trigger="click"
    placement="bottomRight"
    :arrow="false"
    :styles="{ container: { padding: '8px' } }"
  >
    <template #content>
      <div class="git-panel">
        <div class="git-row">
          <span class="git-k">{{ t("gitBranch") }}</span>
          <strong>{{ status.branch }}</strong>
        </div>
        <div class="git-row">
          <span class="git-k">{{ t("gitWorkingTree") }}</span>
          <span :class="status.dirty ? 'is-dirty' : 'is-clean'">
            {{ status.dirty ? t("gitDirty") : t("gitClean") }}
          </span>
        </div>
      </div>
    </template>
    <button
      v-if="variant === 'pill'"
      type="button"
      class="git-pill"
      :class="{ 'is-dirty': status.dirty }"
      :title="title"
    >
      <span class="git-pill-dot" />
      <span>{{ pillLabel }}</span>
    </button>
    <a-button
      v-else
      type="text"
      class="git-icon-btn"
      :class="{ 'is-dirty': status.dirty }"
      :title="title"
    >
      <template #icon><BranchesOutlined /></template>
    </a-button>
  </a-popover>
  <button
    v-else-if="variant === 'pill'"
    type="button"
    class="git-pill is-muted"
    :title="title"
  >
    <span class="git-pill-dot" />
    <span>{{ pillLabel }}</span>
  </button>
  <a-button
    v-else
    type="text"
    class="git-icon-btn is-muted"
    :title="title"
  >
    <template #icon><BranchesOutlined /></template>
  </a-button>
</template>

<style scoped>
.git-pill {
  height: 26px;
  margin: 0;
  padding: 0 8px;
  border: 1px solid var(--line);
  border-radius: var(--radius, 3px);
  background: var(--panel, #fff);
  color: var(--ink-soft);
  font: inherit;
  font-size: 13px;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  gap: 2px;
  cursor: pointer;
  line-height: 1;
  max-width: 180px;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}
.git-pill > span:last-child {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.git-pill:hover {
  border-color: var(--teal-mid, var(--teal));
  background: var(--teal-soft);
  color: var(--teal);
}
.git-pill.is-dirty {
  border-color: color-mix(in srgb, #d97706 40%, var(--line));
  color: #d97706;
}
.git-pill.is-dirty .git-pill-dot {
  background: #d97706;
}
.git-pill.is-muted {
  color: var(--muted);
  opacity: 0.85;
}
.git-pill-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--running, #149a6a);
  flex: none;
}
.git-icon-btn {
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
.git-icon-btn.is-dirty {
  color: #d97706;
}
.git-icon-btn.is-muted {
  color: var(--muted);
  opacity: 0.7;
}
.git-panel {
  width: 300px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 12px;
}
.git-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}
.git-k {
  color: var(--muted);
  font-weight: 600;
}
.is-dirty {
  color: #d97706;
  font-weight: 600;
}
.is-clean {
  color: #059669;
  font-weight: 600;
}
</style>
