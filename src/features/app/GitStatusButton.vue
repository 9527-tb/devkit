<!--
  标题栏 Git 状态图标 + Popover 详情（只读）。
-->
<script setup>
import { computed } from "vue";
import { BranchesOutlined } from "@antdv-next/icons";

const props = defineProps({
  t: { type: Function, required: true },
  status: { type: Object, default: null },
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
    <a-button
      type="text"
      class="git-icon-btn"
      :class="{ 'is-dirty': status.dirty }"
      :title="title"
    >
      <template #icon><BranchesOutlined /></template>
    </a-button>
  </a-popover>
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
