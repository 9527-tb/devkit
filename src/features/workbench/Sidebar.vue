<!--
  侧栏：分组项目列表、筛选。对齐原型（无项目行 ⋯ 菜单）。
  对应 DESIGN.md §12.2 Sidebar
-->
<script setup>
import { computed, ref } from "vue";
import { Empty } from "antdv-next";
import { ReloadOutlined, SearchOutlined } from "@antdv-next/icons";
import { makeProjectKey } from "../../shared/projectKey.js";

// DONE(fe-sidebar): 侧栏分组与打开 Tab — DESIGN §12.2
// DONE(ux-sidebar-filter): 侧栏筛选 — DESIGN §10 / 原型

const props = defineProps({
  t: { type: Function, required: true },
  projects: { type: Array, default: () => [] },
  grouped: { type: Array, default: () => [] },
  selectedPath: { type: String, default: "" },
  selectedKind: { type: String, default: "" },
  projectProcs: { type: Function, required: true },
});

defineEmits(["scan", "select"]);

const { t } = props;
const emptySimpleImage = Empty.PRESENTED_IMAGE_SIMPLE;
const filterText = ref("");

const filteredGrouped = computed(() => {
  const q = filterText.value.trim().toLowerCase();
  if (!q) return props.grouped;
  return props.grouped
    .map(({ group, items }) => ({
      group,
      items: items.filter(
        (p) =>
          p.name.toLowerCase().includes(q) ||
          p.path.toLowerCase().includes(q) ||
          p.kind.toLowerCase().includes(q),
      ),
    }))
    .filter((g) => g.items.length);
});

function projectKey(project) {
  return makeProjectKey(project.path, project.kind);
}

function isSelected(project) {
  return project.path === props.selectedPath && project.kind === props.selectedKind;
}

function metaText(project) {
  const n = props.projectProcs(project.path, project.kind).length;
  return n ? t("instanceCount", { n }) : t("notRunning");
}
</script>

<template>
  <aside class="sidebar">
    <div class="side-head">
      <div>
        <h2>{{ t("projectList") }}</h2>
        <div class="meta">
          {{ t("projectsCount", { n: projects.length, g: grouped.length }) }}
        </div>
      </div>
      <button type="button" class="icon-btn" :title="t('analyze')" @click="$emit('scan')">
        <ReloadOutlined />
      </button>
    </div>

    <div class="side-search">
      <a-input
        v-model:value="filterText"
        allow-clear
        :placeholder="t('filterProjects')"
      >
        <template #prefix>
          <SearchOutlined class="search-icon" />
        </template>
      </a-input>
    </div>

    <div class="side-list">
      <a-empty
        v-if="!projects.length"
        :image="emptySimpleImage"
        :description="t('selectProjectHint')"
      />

      <section v-for="{ group, items } in filteredGrouped" :key="group" class="group">
        <div class="group-h">
          <span class="name">{{ group }}</span>
          <span class="count">{{ items.length }}</span>
        </div>

        <div
          v-for="project in items"
          :key="projectKey(project)"
          class="proj"
          :class="{ active: isSelected(project) }"
          @click="$emit('select', project)"
        >
          <span
            class="dot"
            :class="{ on: projectProcs(project.path, project.kind).length }"
          />
          <span class="pname">{{ project.name }}</span>
          <span
            class="pmeta"
            :class="{ running: projectProcs(project.path, project.kind).length }"
          >
            {{ metaText(project) }}
          </span>
        </div>
      </section>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  --sidebar-bg: color-mix(in srgb, var(--panel, #fff) 92%, white);
  min-height: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--line, #d7e0dc);
  background: var(--sidebar-bg);
}
.side-head {
  flex: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px 8px;
  background: var(--sidebar-bg);
  position: relative;
  z-index: 2;
}
.side-head h2 {
  margin: 0;
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.02em;
}
.side-head .meta {
  margin-top: 2px;
  color: var(--muted, #6b7a76);
  font-size: 10px;
  font-weight: 500;
}
.icon-btn {
  width: 26px;
  height: 26px;
  border: 0;
  border-radius: var(--radius);
  display: grid;
  place-items: center;
  color: var(--muted, #6b7a76);
  background: transparent;
  cursor: pointer;
  font-size: 13px;
}
.icon-btn:hover {
  background: var(--nav-hover, #e8f1ed);
  color: var(--ink, #15201e);
}
.side-search {
  flex: none;
  padding: 0 8px 8px;
  background: var(--sidebar-bg);
  position: relative;
  z-index: 2;
  /* 盖住列表滚动顶沿 1px 发丝缝 */
  box-shadow: 0 1px 0 0 var(--sidebar-bg);
}
.side-search :deep(.ant-input-affix-wrapper) {
  width: 100%;
  border-radius: var(--radius);
  border-color: transparent;
  background: var(--surface-tint, #e8efec);
}
.side-search :deep(.ant-input-affix-wrapper:hover),
.side-search :deep(.ant-input-affix-wrapper-focused) {
  border-color: color-mix(in srgb, var(--teal, #0f766e) 40%, var(--line, #d7e0dc));
  background: #fff;
}
.side-search :deep(.ant-input) {
  background: transparent;
}
.side-search .search-icon {
  color: var(--muted, #6b7a76);
}
.side-list {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 0 8px 12px;
}
.group-h {
  display: flex;
  align-items: center;
  gap: 6px;
  /* top:-1px + 补 1px padding，封住 sticky 顶边亚像素缝 */
  padding: 5px 0 5px;
  position: sticky;
  top: -1px;
  z-index: 1;
  background: var(--sidebar-bg);
  box-shadow:
    0 -1px 0 0 var(--sidebar-bg),
    0 1px 0 0 var(--sidebar-bg);
}
.group-h .name {
  font-size: 11px;
  font-weight: 700;
  color: var(--ink-soft, #30433e);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
.group-h .count {
  color: var(--muted, #6b7a76);
  font-size: 10px;
  font-weight: 600;
}
.proj {
  display: grid;
  grid-template-columns: 6px minmax(0, 1fr) auto;
  align-items: center;
  gap: 7px;
  height: 28px;
  margin: 1px 0;
  padding: 0 6px 0 8px;
  border-radius: var(--radius);
  cursor: pointer;
  transition: background 0.12s;
  box-sizing: border-box;
}
.proj:hover {
  background: var(--nav-hover, #e8f1ed);
}
.proj.active {
  background: var(--teal-soft, #e8f5f2);
}
.proj.active .pname {
  color: var(--teal, #0f766e);
  font-weight: 700;
}
.dot {
  width: 6px;
  height: 6px;
  min-width: 6px;
  min-height: 6px;
  border-radius: 50%;
  box-sizing: border-box;
  background: var(--line-hover, #b9cdc5);
  box-shadow: none;
  transition: background 0.12s ease;
}
.dot.on {
  background: var(--running, #149a6a);
}
.pname {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-strong, #24322e);
  line-height: 28px;
}
.pmeta {
  font-size: 10px;
  font-weight: 600;
  color: var(--muted, #6b7a76);
  white-space: nowrap;
  line-height: 28px;
}
.pmeta.running {
  color: var(--teal, #0f766e);
}
</style>
