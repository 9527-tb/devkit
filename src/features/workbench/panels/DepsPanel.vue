<!--
  依赖面板：dep-bar + a-table children 树表。
  对应 DESIGN.md §8 panel.deps
-->
<script setup>
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { message } from "antdv-next";
import { createTranslator } from "../../../i18n/index.js";
import { locale, previewMode } from "../../../stores/settings.js";

const props = defineProps({
  project: { type: Object, required: true },
  tree: { type: Array, default: () => [] },
  count: { type: Number, default: 0 },
  loading: { type: Boolean, default: false },
});

defineEmits(["refresh"]);

const t = createTranslator(locale);

const outdatedLoading = ref(false);
const outdatedRows = ref([]);
const showOutdated = ref(false);

const isNodeProject = computed(() => props.project?.kind === "node");

const outdatedColumns = computed(() => [
  { title: t("colDependency"), dataIndex: "name", key: "name" },
  { title: t("outdatedCurrent"), dataIndex: "current", key: "current", width: 100 },
  { title: t("outdatedWanted"), dataIndex: "wanted", key: "wanted", width: 100 },
  { title: t("outdatedLatest"), dataIndex: "latest", key: "latest", width: 100 },
]);

async function checkOutdated() {
  if (!isNodeProject.value) return;
  if (previewMode.value) {
    message.info(t("previewNeedTauri"));
    return;
  }
  outdatedLoading.value = true;
  showOutdated.value = true;
  try {
    outdatedRows.value = await invoke("check_outdated_deps", {
      path: props.project.path,
      kind: props.project.kind,
    });
    if (!outdatedRows.value.length) {
      message.success(t("outdatedNone"));
    }
  } catch (error) {
    message.error(String(error));
    outdatedRows.value = [];
  } finally {
    outdatedLoading.value = false;
  }
}

const title = computed(() => {
  if (props.project.kind === "maven") return t("depsTitleMaven");
  if (props.project.kind === "cargo") return t("depsTitleCargo");
  if (props.project.kind === "gradle") return t("depsTitleGradle");
  return t("depsTitleNode");
});

const columns = computed(() => [
  { title: t("colDependency"), dataIndex: "name", key: "name" },
  { title: t("colVersion"), dataIndex: "version", key: "version", width: 120 },
  { title: t("colScope"), dataIndex: "scope", key: "scope", width: 120 },
]);

/** 分组节点用 i18n 计数；兼容旧数据里写死的「N 项 / N 根」 */
function formatVersion(record) {
  if (record?.scope === "group") {
    const n =
      Array.isArray(record.children) && record.children.length
        ? record.children.length
        : parseLegacyCount(record.version);
    return t(record.key === "mvn-tree" ? "depRoots" : "depItems", { n });
  }
  return record?.version || "—";
}

function parseLegacyCount(version) {
  const m = String(version || "").match(/^(\d+)/);
  return m ? Number(m[1]) : 0;
}
</script>

<template>
  <div class="dep-pane">
    <div class="dep-bar">
      <span>{{ title }}</span>
      <div class="dep-bar-actions">
        <button
          v-if="isNodeProject"
          type="button"
          class="dep-link"
          :disabled="outdatedLoading"
          @click="checkOutdated"
        >
          {{ outdatedLoading ? t("refreshing") : t("checkOutdated") }}
        </button>
        <button
          type="button"
          class="dep-link"
          :disabled="loading"
          @click="$emit('refresh')"
        >
          {{ loading ? t("refreshing") : t("refreshDeps") }}
        </button>
        <span class="dep-count">{{ t("depItems", { n: count }) }}</span>
      </div>
    </div>
    <p v-if="showOutdated && isNodeProject" class="outdated-note">{{ t("outdatedReadOnly") }}</p>
    <div v-if="showOutdated && outdatedRows.length" class="outdated-wrap">
      <a-table
        class="dep-table"
        size="small"
        :pagination="false"
        :columns="outdatedColumns"
        :data-source="outdatedRows"
        row-key="name"
        :locale="{ emptyText: t('outdatedEmpty') }"
      />
    </div>
    <div class="dep-table-wrap">
      <a-table
        class="dep-table"
        size="small"
        :pagination="false"
        :columns="columns"
        :data-source="tree"
        :loading="loading"
        :default-expand-all-rows="true"
        children-column-name="children"
        row-key="key"
        :locale="{ emptyText: t('depsEmpty') }"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'name'">
            <span class="pkg" :class="{ group: record.scope === 'group' }">
              {{ record.name }}
            </span>
          </template>
          <template v-else-if="column.key === 'version'">
            <span class="ver">{{ formatVersion(record) }}</span>
          </template>
          <template v-else-if="column.key === 'scope'">
            <span v-if="record.scope && record.scope !== 'group'" class="scope">
              {{ record.scope }}
            </span>
          </template>
        </template>
      </a-table>
    </div>
  </div>
</template>

<style scoped>
.dep-pane {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--line, #d7e0dc);
  border-radius: var(--radius);
  background: var(--panel, #fff);
  overflow: hidden;
}
.dep-bar {
  flex: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-bottom: 1px solid var(--line, #d7e0dc);
  background: var(--input-bg, #f4f8f6);
  font-size: 12px;
  font-weight: 600;
}
.dep-bar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
.dep-link {
  border: 0;
  background: none;
  padding: 0;
  color: var(--teal, #0f766e);
  font: inherit;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}
.dep-link:disabled {
  opacity: 0.5;
  cursor: default;
}
.dep-count {
  color: var(--muted, #6b7a76);
  font-size: 11px;
  font-weight: 700;
}
.dep-table-wrap {
  flex: 1;
  min-height: 0;
  overflow: auto;
}
.dep-table :deep(.ant-table) {
  background: transparent;
  font-size: 12px;
}
.dep-table :deep(.ant-table-thead > tr > th) {
  background: var(--surface-muted, #f5f7f6);
  color: var(--muted, #6b7a76);
  font-size: 11px;
  font-weight: 700;
  border-bottom: 1px solid var(--line-soft, #eef3f0);
  padding: 8px 10px;
}
.dep-table :deep(.ant-table-tbody > tr > td) {
  border-bottom: 1px solid var(--line-soft, #eef3f0);
  padding: 6px 10px;
}
.dep-table :deep(.ant-table-container),
.dep-table :deep(.ant-table),
.dep-table :deep(table) {
  border-radius: 0 !important;
}
.pkg {
  font-weight: 600;
  color: var(--ink-strong, #24322e);
}
.pkg.group {
  color: var(--teal, #0f766e);
}
.ver {
  color: var(--muted, #6b7a76);
  font-family: "IBM Plex Mono", ui-monospace, monospace;
  font-size: 11px;
}
.scope {
  display: inline-block;
  padding: 1px 6px;
  border-radius: var(--radius);
  background: var(--surface-hover, #f3f7f5);
  color: var(--muted, #6b7a76);
  font-size: 10px;
  font-weight: 700;
}
.outdated-note {
  margin: 0;
  padding: 6px 10px;
  font-size: 11px;
  color: var(--muted, #6b7a76);
  background: var(--input-bg, #f4f8f6);
  border-bottom: 1px solid var(--line-soft, #eef3f0);
}
.outdated-wrap {
  flex: none;
  max-height: 200px;
  overflow: auto;
  border-bottom: 1px solid var(--line, #d7e0dc);
}
</style>
