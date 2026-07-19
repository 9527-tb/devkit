<script setup>
import { computed, ref } from "vue";
import Sidebar from "../features/workbench/Sidebar.vue";
import ProjectTabs from "../features/workbench/ProjectTabs.vue";
import ProjectHeader from "../features/workbench/ProjectHeader.vue";
import CloseTabConfirm from "../features/workbench/CloseTabConfirm.vue";
import LogsPanel from "../features/workbench/panels/LogsPanel.vue";
import DepsPanel from "../features/workbench/panels/DepsPanel.vue";
import MonitorPanel from "../features/workbench/panels/MonitorPanel.vue";
import { useWorkbench } from "../features/workbench/useWorkbench.js";
import { logWrap } from "../stores/settings.js";
import "../features/workbench/workbench.css";

const props = defineProps({
  t: { type: Function, required: true },
});

const {
  projects,
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
  currentRunSummary,
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
  openInEditor,
  openInTerminal,
  t,
} = useWorkbench(props.t);

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
            <ProjectHeader
              :project="current"
              :t="t"
              :show-instance-select="showInstanceSelect"
              :selected-pid="selectedPid"
              :instance-options="instanceOptions"
              :actions="actionList"
              :display-action="displayAction"
              :has-running="!!currentProcesses.length"
              :stop-label="t('stop')"
              @update:selected-pid="selectedPid = $event"
              @open-editor="openInEditor(current)"
              @open-terminal="openInTerminal(current)"
              @run="(action) => run(current, action)"
              @stop="
                showInstanceSelect && selectedPid
                  ? stopInstance(current, selectedPid)
                  : stop(current)
              "
            />

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
                :run-summary="currentRunSummary"
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

