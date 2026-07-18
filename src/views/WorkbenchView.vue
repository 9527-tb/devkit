<script setup>
import { computed, ref } from "vue";
import Topbar from "../features/workbench/Topbar.vue";
import Sidebar from "../features/workbench/Sidebar.vue";
import ProjectTabs from "../features/workbench/ProjectTabs.vue";
import ProjectHeader from "../features/workbench/ProjectHeader.vue";
import ActionBar from "../features/workbench/ActionBar.vue";
import CloseTabConfirm from "../features/workbench/CloseTabConfirm.vue";
import LogsPanel from "../features/workbench/panels/LogsPanel.vue";
import DepsPanel from "../features/workbench/panels/DepsPanel.vue";
import MonitorPanel from "../features/workbench/panels/MonitorPanel.vue";
import { useWorkbench } from "../features/workbench/useWorkbench.js";
import { logWrap } from "../stores/settings.js";
import "../features/workbench/workbench.css";

// DONE(fe-workbench-view): 布局对齐原型 — 顶栏 / 侧栏+主区；Tab 在主区顶部

const props = defineProps({
  t: { type: Function, required: true },
});
const emit = defineEmits(["open-settings", "open-tools"]);

const {
  root,
  projects,
  loading,
  previewMode,
  workspaceHistoryMenu,
  dropdownPopupContainer,
  chooseDirectory,
  scan,
  openTabs,
  activeTabKey,
  tabIsRunning,
  onActivateTab,
  requestCloseTab,
  grouped,
  selectedPath,
  selectedKind,
  projectProcs,
  selectProject,
  current,
  showInstanceSelect,
  selectedPid,
  instanceOptions,
  runtimePill,
  actionList,
  displayAction,
  currentProcesses,
  run,
  stop,
  stopInstance,
  activePanel,
  switchPanel,
  logs,
  clearLogs,
  dependencyTree,
  dependencyCount,
  depLoading,
  refreshDependencies,
  closeConfirmOpen,
  closeConfirmTab,
  closeConfirmMode,
  closeConfirmBatch,
  closeConfirmRunningCount,
  confirmCloseTab,
  cancelCloseConfirm,
  onTabContextAction,
  t,
} = useWorkbench(props.t, emit);

const monitorRef = ref(null);
const selectedProcess = computed(
  () =>
    currentProcesses.value.find((p) => Number(p.pid) === Number(selectedPid.value)) ||
    currentProcesses.value[0] ||
    null,
);

function refreshMonitor() {
  monitorRef.value?.refresh?.();
}
</script>

<template>
  <div class="workbench">
    <Topbar
      :t="t"
      :root="root"
      :loading="loading"
      :preview-mode="previewMode"
      :workspace-history-menu="workspaceHistoryMenu"
      :dropdown-popup-container="dropdownPopupContainer"
      @choose-directory="chooseDirectory"
      @scan="scan"
      @open-settings="emit('open-settings')"
      @open-tools="emit('open-tools')"
    />

    <div class="body">
      <Sidebar
        :t="t"
        :projects="projects"
        :grouped="grouped"
        :selected-path="selectedPath"
        :selected-kind="selectedKind"
        :project-procs="projectProcs"
        @scan="scan"
        @select="selectProject"
      />

      <section class="main">
        <ProjectTabs
          :tabs="openTabs"
          :active-key="activeTabKey"
          :is-running="tabIsRunning"
          @activate="onActivateTab"
          @close="requestCloseTab"
          @jump="onActivateTab"
          @context-action="onTabContextAction"
        />

        <template v-if="current">
          <div class="main-body">
            <ProjectHeader :project="current" :runtime-label="runtimePill">
              <template #actions>
                <ActionBar
                  :actions="actionList"
                  :display-action="displayAction"
                  :has-running="!!currentProcesses.length"
                  :show-instance-select="showInstanceSelect"
                  :selected-pid="selectedPid"
                  :instance-options="instanceOptions"
                  :stop-label="t('stop')"
                  @update:selected-pid="selectedPid = $event"
                  @run="(action) => run(current, action)"
                  @stop="
                    showInstanceSelect && selectedPid
                      ? stopInstance(current, selectedPid)
                      : stop(current)
                  "
                />
              </template>
            </ProjectHeader>

            <div class="panels">
              <div class="tabs">
                <div class="tab-list">
                  <button
                    type="button"
                    class="tab"
                    :class="{ active: activePanel === 'logs' }"
                    @click="switchPanel('logs')"
                  >
                    {{ t("logs") }}
                  </button>
                  <button
                    type="button"
                    class="tab"
                    :class="{ active: activePanel === 'dependencies' }"
                    @click="switchPanel('dependencies')"
                  >
                    {{ t("deps") }}
                  </button>
                  <button
                    type="button"
                    class="tab"
                    :class="{ active: activePanel === 'monitor' }"
                    @click="switchPanel('monitor')"
                  >
                    {{ t("monitor") }}
                  </button>
                </div>
                <div v-show="activePanel === 'logs'" class="tab-tools">
                  <a-checkbox v-model:checked="logWrap" class="log-wrap-check">
                    {{ t("wrap") }}
                  </a-checkbox>
                </div>
                <div v-show="activePanel === 'monitor'" class="tab-tools">
                  <button type="button" class="panel-tool-link" @click="refreshMonitor">
                    {{ t("refresh") }}
                  </button>
                </div>
              </div>

              <LogsPanel
                v-show="activePanel === 'logs'"
                :key="`${selectedPath}::${selectedKind}`"
                :t="t"
                :project-name="current.name"
                :lines="logs"
                @clear="clearLogs"
              />
              <DepsPanel
                v-show="activePanel === 'dependencies'"
                :project="current"
                :tree="dependencyTree"
                :count="dependencyCount"
                :loading="depLoading"
                @refresh="refreshDependencies"
              />
              <MonitorPanel
                ref="monitorRef"
                v-show="activePanel === 'monitor'"
                :project="current"
                :running="!!currentProcesses.length"
                :pid="selectedPid"
                :action="selectedProcess?.action || ''"
                :runtime-label="runtimePill"
              />
            </div>
          </div>
        </template>

        <div v-else class="main-empty">
          {{ t("selectFromLeft") }}
        </div>
      </section>
    </div>

    <CloseTabConfirm
      v-model:open="closeConfirmOpen"
      :mode="closeConfirmMode"
      :project-name="closeConfirmTab?.name || ''"
      :total-count="closeConfirmBatch.length"
      :running-count="closeConfirmRunningCount"
      @confirm="confirmCloseTab"
      @cancel="cancelCloseConfirm"
    />
  </div>
</template>
