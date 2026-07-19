<!--
  标题栏中央：当前项目名 + Popover（选择新项目 / 历史清单）
-->
<script setup>
import { computed, ref } from "vue";
import {
  FolderOpenOutlined,
  HistoryOutlined,
  DownOutlined,
} from "@antdv-next/icons";
import {
  root,
  workspaceHistory,
  workspaceBasename,
  historyLabel,
  clearWorkspaceHistory,
} from "../../stores/workspace.js";
import { openTabs, activeKey } from "../../stores/projectTabs.js";
import { settingsPage } from "../../stores/settings.js";
import { toolsPage } from "../../stores/tools.js";
import {
  chooseNewWorkspace,
  openHistoryWorkspace,
} from "../workbench/useWorkspacePicker.js";

const props = defineProps({
  t: { type: Function, required: true },
});

const open = ref(false);

const displayName = computed(() => {
  const tab = openTabs.value.find((item) => item.key === activeKey.value);
  if (tab?.name) return tab.name;
  if (root.value) return workspaceBasename(root.value);
  return props.t("noProjectSelected");
});

const fullTitle = computed(() => {
  const tab = openTabs.value.find((item) => item.key === activeKey.value);
  if (tab?.path) return tab.path;
  return root.value || "";
});

function goWorkbench() {
  settingsPage.value = false;
  toolsPage.value = false;
}

async function onChooseNew() {
  open.value = false;
  goWorkbench();
  await chooseNewWorkspace(props.t);
}

function onOpenHistory(path) {
  open.value = false;
  goWorkbench();
  openHistoryWorkspace(path, props.t);
}

function onClearHistory() {
  clearWorkspaceHistory();
}
</script>

<template>
  <a-popover
    v-model:open="open"
    trigger="click"
    placement="bottom"
    :arrow="false"
    :styles="{ container: { padding: '8px' } }"
    :classes="{ root: 'workspace-switcher-popover' }"
  >
    <template #content>
      <div class="ws-switcher">
        <div class="ws-switcher-section">
          <button type="button" class="ws-switcher-item" @click="onChooseNew">
            <FolderOpenOutlined class="ws-switcher-icon" />
            <span>{{ t("chooseNewProject") }}</span>
          </button>
        </div>

        <div class="ws-switcher-divider" />

        <div class="ws-switcher-section">
          <div class="ws-switcher-label">
            <HistoryOutlined />
            {{ t("workspaceHistory") }}
          </div>
          <div v-if="!workspaceHistory.length" class="ws-switcher-empty">
            {{ t("workspaceHistoryEmpty") }}
          </div>
          <button
            v-for="path in workspaceHistory"
            :key="path"
            type="button"
            class="ws-switcher-item"
            :class="{ active: path === root }"
            :title="path"
            @click="onOpenHistory(path)"
          >
            <span class="ws-switcher-history-name">{{ historyLabel(path) }}</span>
          </button>
          <button
            v-if="workspaceHistory.length"
            type="button"
            class="ws-switcher-item danger"
            @click="onClearHistory"
          >
            {{ t("clearWorkspaceHistory") }}
          </button>
        </div>
      </div>
    </template>

    <a-button
      type="text"
      class="ws-switcher-trigger"
      :title="fullTitle || t('chooseNewProject')"
    >
      <span class="ws-switcher-trigger-text">{{ displayName }}</span>
      <DownOutlined class="ws-switcher-caret" />
    </a-button>
  </a-popover>
</template>

<style scoped>
.ws-switcher-trigger {
  max-width: min(360px, 42vw);
  height: var(--titlebar-control-h, 28px) !important;
  padding: 0 10px !important;
  display: inline-flex !important;
  align-items: center;
  gap: 6px;
  font-weight: 600;
  color: var(--ink) !important;
  -webkit-app-region: no-drag;
  app-region: no-drag;
}

.ws-switcher-trigger-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  min-width: 0;
}

.ws-switcher-caret {
  flex: none;
  font-size: 10px;
  opacity: 0.55;
}

.ws-switcher {
  width: 300px;
  padding: 0;
}

.ws-switcher-section {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0;
}

.ws-switcher-divider {
  height: 1px;
  margin: 6px 8px;
  background: var(--line);
}

.ws-switcher-label {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px 6px;
  color: var(--muted);
  font-size: 11px;
  font-weight: 600;
}

.ws-switcher-empty {
  padding: 6px 10px 8px;
  color: var(--muted);
  font-size: 12px;
}

.ws-switcher-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  padding: 7px 10px;
  border: 0;
  border-radius: var(--radius);
  background: transparent;
  color: var(--ink-soft);
  font: inherit;
  font-size: 12px;
  font-weight: 600;
  text-align: left;
  cursor: pointer;
}

.ws-switcher-item:hover {
  background: var(--nav-hover);
  color: var(--ink);
}

.ws-switcher-item.active {
  background: var(--teal-soft);
  color: var(--teal);
}

.ws-switcher-item.danger {
  color: #cf1322;
  font-weight: 500;
}

.ws-switcher-icon {
  flex: none;
  font-size: 13px;
}

.ws-switcher-history-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>

<style>
/* antdv-next：内边距在 *-container 上；styles.container 为主，此处兜底 */
.workspace-switcher-popover.ant-popover .ant-popover-container,
.workspace-switcher-popover .ant-popover-container {
  padding: 8px !important;
}
</style>
