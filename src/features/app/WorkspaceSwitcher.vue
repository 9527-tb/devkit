<!--
  标题栏中央：当前工作区 / 多根会话 + Popover（切换 / 添加 / 已挂载 / 历史）
-->
<script setup>
import { computed, ref } from "vue";
import {
  FolderOpenOutlined,
  FolderAddOutlined,
  HistoryOutlined,
  DownOutlined,
  CloseOutlined,
} from "@antdv-next/icons";
import {
  root,
  workspaceHistory,
  workspaceRoots,
  workspaceBasename,
  historyLabel,
  clearWorkspaceHistory,
  removeWorkspaceRoot,
  setRootEnabled,
  sessionDisplayName,
  enabledWorkspaceRoots,
} from "../../stores/workspace.js";
import { openTabs, activeKey } from "../../stores/projectTabs.js";
import { goWorkbench } from "../../stores/appNav.js";
import {
  chooseNewWorkspace,
  addWorkspaceFolder,
  openHistoryWorkspace,
} from "../workbench/useWorkspacePicker.js";

const props = defineProps({
  t: { type: Function, required: true },
});

const open = ref(false);

const displayName = computed(() => {
  const tab = openTabs.value.find((item) => item.key === activeKey.value);
  if (tab?.name) return tab.name;
  const session = sessionDisplayName(props.t);
  if (session) return session;
  return props.t("noProjectSelected");
});

const fullTitle = computed(() => {
  const tab = openTabs.value.find((item) => item.key === activeKey.value);
  if (tab?.path) return tab.path;
  return enabledWorkspaceRoots.value.map((r) => r.path).join("\n") || root.value || "";
});

async function onChooseNew() {
  open.value = false;
  goWorkbench();
  await chooseNewWorkspace(props.t);
}

async function onAddRoot() {
  open.value = false;
  goWorkbench();
  await addWorkspaceFolder(props.t);
}

function onOpenHistory(path) {
  open.value = false;
  goWorkbench();
  openHistoryWorkspace(path, props.t);
}

function onClearHistory() {
  clearWorkspaceHistory();
}

function onToggleRoot(rootItem, checked) {
  setRootEnabled(rootItem.id, checked);
}

function onRemoveRoot(rootItem, e) {
  e?.stopPropagation?.();
  removeWorkspaceRoot(rootItem.id);
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
          <button type="button" class="ws-switcher-item" @click="onAddRoot">
            <FolderAddOutlined class="ws-switcher-icon" />
            <span>{{ t("addWorkspaceRoot") }}</span>
          </button>
        </div>

        <template v-if="workspaceRoots.length">
          <div class="ws-switcher-divider" />
          <div class="ws-switcher-section">
            <div class="ws-switcher-label">{{ t("mountedRoots") }}</div>
            <div
              v-for="item in workspaceRoots"
              :key="item.id"
              class="ws-switcher-root-row"
            >
              <a-checkbox
                :checked="item.enabled"
                @change="(e) => onToggleRoot(item, e?.target?.checked ?? !item.enabled)"
              >
                <span class="ws-switcher-history-name" :title="item.path">
                  {{ item.label || workspaceBasename(item.path) }}
                </span>
              </a-checkbox>
              <button
                type="button"
                class="ws-switcher-remove"
                :title="t('removeWorkspaceRoot')"
                @click="onRemoveRoot(item, $event)"
              >
                <CloseOutlined />
              </button>
            </div>
          </div>
        </template>

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

.ws-switcher-root-row {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 6px 2px 10px;
}

.ws-switcher-root-row :deep(.ant-checkbox-wrapper) {
  flex: 1;
  min-width: 0;
  font-size: 12px;
  font-weight: 600;
}

.ws-switcher-remove {
  flex: none;
  width: 22px;
  height: 22px;
  border: 0;
  border-radius: var(--radius);
  background: transparent;
  color: var(--muted);
  cursor: pointer;
  display: grid;
  place-items: center;
  font-size: 10px;
}

.ws-switcher-remove:hover {
  background: var(--nav-hover);
  color: #cf1322;
}
</style>

<style>
.workspace-switcher-popover.ant-popover .ant-popover-container,
.workspace-switcher-popover .ant-popover-container {
  padding: 8px !important;
}
</style>
