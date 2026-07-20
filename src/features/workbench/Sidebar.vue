<!--
  侧栏：多根 → Kind 分组（可折叠），无多选。
-->
<script setup>
import { computed, ref } from "vue";
import { Empty } from "antdv-next";
import {
  CaretRightOutlined,
  ReloadOutlined,
  SearchOutlined,
} from "@antdv-next/icons";
import { makeProjectKey } from "../../shared/projectKey.js";

const props = defineProps({
  t: { type: Function, required: true },
  projects: { type: Array, default: () => [] },
  /** [{ rootId, rootLabel, rootPath, kinds: [{ group, items }], count }] */
  grouped: { type: Array, default: () => [] },
  selectedPath: { type: String, default: "" },
  selectedKind: { type: String, default: "" },
  projectProcs: { type: Function, required: true },
  rootCount: { type: Number, default: 0 },
});

defineEmits(["scan", "select"]);

const { t } = props;
const emptySimpleImage = Empty.PRESENTED_IMAGE_SIMPLE;
const filterText = ref("");
/** 折叠的 Kind 分组 key：`${rootId}::${group}` */
const collapsedKinds = ref(new Set());

const filteredGrouped = computed(() => {
  const q = filterText.value.trim().toLowerCase();
  return props.grouped
    .map((root) => {
      const kinds = (root.kinds || [])
        .map(({ group, items }) => ({
          group,
          items: q
            ? items.filter(
                (p) =>
                  p.name.toLowerCase().includes(q) ||
                  p.path.toLowerCase().includes(q) ||
                  p.kind.toLowerCase().includes(q),
              )
            : items,
        }))
        .filter((g) => g.items.length);
      const count = kinds.reduce((n, g) => n + g.items.length, 0);
      return { ...root, kinds, count };
    })
    .filter((r) => r.count > 0);
});

const kindGroupCount = computed(() =>
  filteredGrouped.value.reduce((n, r) => n + r.kinds.length, 0),
);

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

function kindKey(root, group) {
  return `${root.rootId || root.rootPath || "_"}::${group}`;
}

function isKindCollapsed(root, group) {
  return collapsedKinds.value.has(kindKey(root, group));
}

function toggleKind(root, group) {
  const key = kindKey(root, group);
  const next = new Set(collapsedKinds.value);
  if (next.has(key)) next.delete(key);
  else next.add(key);
  collapsedKinds.value = next;
}
</script>

<template>
  <aside class="sidebar">
    <div class="side-head">
      <div>
        <h2>{{ t("explorer") }}</h2>
        <div class="meta">
          {{
            t("projectsCountRoots", {
              n: projects.length,
              r: rootCount || grouped.length,
              g: kindGroupCount,
            })
          }}
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
        size="small"
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

      <section
        v-for="root in filteredGrouped"
        :key="root.rootId || root.rootPath || '_'"
        class="root-block"
      >
        <div v-if="root.rootLabel || grouped.length > 1" class="root-h">
          <span class="name">{{ root.rootLabel || root.rootPath }}</span>
          <span class="count">{{ root.count }}</span>
        </div>

        <section v-for="{ group, items } in root.kinds" :key="`${root.rootId}-${group}`" class="group">
          <div class="group-h" @click="toggleKind(root, group)">
            <CaretRightOutlined
              class="group-caret"
              :class="{ open: !isKindCollapsed(root, group) }"
            />
            <span class="name">{{ group }}</span>
            <span class="count">{{ items.length }}</span>
          </div>

          <div v-show="!isKindCollapsed(root, group)" class="group-body">
            <div
              v-for="project in items"
              :key="projectKey(project)"
              class="proj"
              :class="{ active: isSelected(project) }"
              @click="$emit('select', project)"
            >
              <span class="pname">{{ project.name }}</span>
              <span
                class="pmeta"
                :class="{ running: projectProcs(project.path, project.kind).length }"
              >
                {{ metaText(project) }}
              </span>
            </div>
          </div>
        </section>
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
  padding: 0 8px 6px;
  background: var(--sidebar-bg);
  position: relative;
  z-index: 2;
  box-shadow: 0 1px 0 0 var(--sidebar-bg);
}
.side-search :deep(.ant-input-affix-wrapper) {
  width: 100%;
  min-height: 24px;
  padding: 0 7px;
  border-radius: var(--radius);
  border-color: transparent;
  background: var(--surface-tint, #e8efec);
  font-size: 11px;
}
.side-search :deep(.ant-input-affix-wrapper .ant-input) {
  font-size: 11px;
  line-height: 22px;
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
  font-size: 11px;
  color: var(--muted, #6b7a76);
}
.side-list {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 0 8px 12px;
}
.root-h {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 0 4px;
  position: sticky;
  top: -1px;
  z-index: 2;
  background: var(--sidebar-bg);
  box-shadow: 0 -1px 0 0 var(--sidebar-bg), 0 1px 0 0 var(--sidebar-bg);
}
.root-h .name {
  font-size: 12px;
  font-weight: 800;
  color: var(--ink);
}
.root-h .count {
  color: var(--muted);
  font-size: 10px;
  font-weight: 600;
}
.group-h,
.proj {
  height: 28px;
  box-sizing: border-box;
}
.group-h {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 0 6px 0 4px;
  position: sticky;
  top: 22px;
  z-index: 1;
  background: var(--sidebar-bg);
  box-shadow: 0 -1px 0 0 var(--sidebar-bg), 0 1px 0 0 var(--sidebar-bg);
  cursor: pointer;
  user-select: none;
  border-radius: var(--radius);
}
.group-h:hover {
  background: color-mix(in srgb, var(--nav-hover, #e8f1ed) 70%, var(--sidebar-bg));
}
.group-caret {
  flex: none;
  width: 12px;
  font-size: 10px;
  color: var(--muted, #6b7a76);
  transition: transform 0.12s ease;
}
.group-caret.open {
  transform: rotate(90deg);
}
.group-h .name {
  font-size: 12px;
  font-weight: 700;
  color: var(--ink-soft, #30433e);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  line-height: 28px;
}
.group-h .count {
  color: var(--muted, #6b7a76);
  font-size: 10px;
  font-weight: 600;
  line-height: 28px;
}
.proj {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 6px;
  margin: 0;
  /* 相对类型行：caret(12) + gap(4) + 额外缩进(10) */
  padding: 0 6px 0 26px;
  border-radius: var(--radius);
  cursor: pointer;
  transition: background 0.12s;
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
