<!--
  编排计划：左列表 + 图画布（Start/Step/End 连线）。
-->
<script setup>
import { computed, ref, watch } from "vue";
import { PlusOutlined } from "@antdv-next/icons";
import { Modal, message } from "antdv-next";
import PlanFlowCanvas from "./planFlow/PlanFlowCanvas.vue";
import {
  NODE_STEP,
  createEmptyPlanGraph,
  newPlanId,
  newNodeId,
  normalizeRunPlan,
  projectRelPath,
  makeProjectKey,
  validatePlanGraph,
  NODE_START,
  NODE_END,
} from "./runPlans.js";

const props = defineProps({
  open: { type: Boolean, default: false },
  layout: { type: String, default: "page" },
  t: { type: Function, required: true },
  projects: { type: Array, default: () => [] },
  plans: { type: Array, default: () => [] },
  primaryRootPath: { type: String, default: "" },
  runningPlan: { type: Object, default: null },
});

const emit = defineEmits([
  "update:open",
  "execute",
  "stop",
  "delete",
  "save-user",
  "save-workspace",
]);

const name = ref("");
const planId = ref("");
const stopOnError = ref(true);
const nodes = ref([]);
const edges = ref([]);
const selectedId = ref("");

const addOpen = ref(false);
const pickKey = ref("");
const pickAction = ref("");
const pickWait = ref("spawn");

const isPage = computed(() => props.layout === "page");
const isPlanRunning = computed(() => !!(props.runningPlan && props.runningPlan.id));
const isEditingSaved = computed(() => props.plans.some((p) => p.id === planId.value));

const projectByKey = computed(() => {
  const map = new Map();
  for (const p of props.projects || []) {
    map.set(makeProjectKey(p.path, p.kind), p);
  }
  return map;
});

const projectOptions = computed(() =>
  (props.projects || []).map((p) => ({
    value: makeProjectKey(p.path, p.kind),
    label: p.rootLabel ? `${p.name} · ${p.kind} · ${p.rootLabel}` : `${p.name} · ${p.kind}`,
  })),
);

const pickedProject = computed(() => projectByKey.value.get(pickKey.value) || null);

const actionOptions = computed(() => {
  const p = pickedProject.value;
  if (!p) return [];
  if (p.kind === "node") {
    return [
      { value: "install", label: "install" },
      ...(p.scripts || []).map((s) => ({ value: `script:${s}`, label: s })),
    ];
  }
  return (p.scripts || []).map((s) => ({ value: s, label: s }));
});

const graphCheck = computed(() => validatePlanGraph(nodes.value, edges.value));
const canSubmit = computed(() => graphCheck.value.ok);

const summaryText = computed(() => {
  const t = props.t;
  if (!graphCheck.value.ok) {
    const map = {
      needStart: t("runPlanErrNeedStart"),
      needEnd: t("runPlanErrNeedEnd"),
      needSteps: t("runPlanErrNeedSteps"),
      incompleteStep: t("runPlanErrIncomplete"),
      cycle: t("runPlanErrCycle"),
      startDisconnected: t("runPlanErrStartLink"),
      endDisconnected: t("runPlanErrEndLink"),
      unreachableEnd: t("runPlanErrPath"),
    };
    return map[graphCheck.value.reason] || t("runPlanSummaryNeedSteps");
  }
  const steps = nodes.value.filter((n) => n.type === NODE_STEP).length;
  return t("runPlanGraphSummary", { n: steps, e: edges.value.length });
});

function defaultWait(action) {
  return action === "install" || action === "build" ? "complete" : "spawn";
}

function resetForm() {
  planId.value = newPlanId();
  name.value = "";
  stopOnError.value = true;
  selectedId.value = "";
  const g = createEmptyPlanGraph();
  nodes.value = g.nodes;
  edges.value = g.edges;
  if (props.projects?.length) {
    pickKey.value = makeProjectKey(props.projects[0].path, props.projects[0].kind);
  } else {
    pickKey.value = "";
  }
  pickAction.value = "";
  pickWait.value = "spawn";
}

watch(
  () => props.open,
  (v) => {
    if (isPage.value || v) resetForm();
  },
  { immediate: true },
);

watch(pickAction, (a) => {
  if (a) pickWait.value = defaultWait(a);
});

function loadExisting(plan) {
  const n = normalizeRunPlan(plan, plan.source);
  if (!n) {
    message.warning(props.t("runPlanInvalidGraph"));
    return;
  }
  planId.value = n.id;
  name.value = n.name;
  stopOnError.value = n.stopOnError !== false;
  nodes.value = n.nodes.map((x) => ({ ...x, position: { ...x.position } }));
  edges.value = n.edges.map((x) => ({ ...x }));
  selectedId.value = "";
}

function onNewPlan() {
  resetForm();
}

function buildPlan() {
  if (!canSubmit.value) return null;
  return normalizeRunPlan(
    {
      id: planId.value,
      name: name.value || props.t("runPlanUntitled"),
      stopOnError: stopOnError.value,
      rootPath: props.primaryRootPath,
      nodes: nodes.value,
      edges: edges.value,
    },
    "user",
  );
}

function onExecute() {
  const plan = buildPlan();
  if (!plan) return;
  emit("execute", plan);
}

function onSaveUser() {
  const plan = buildPlan();
  if (!plan) return;
  emit("save-user", plan);
}

function onSaveWorkspace() {
  const plan = buildPlan();
  if (!plan) return;
  emit("save-workspace", plan);
}

function openAddStep() {
  if (!props.projects?.length) {
    message.warning(props.t("runPlanNeedScan"));
    return;
  }
  editingStepId.value = "";
  if (!pickKey.value && props.projects[0]) {
    pickKey.value = makeProjectKey(props.projects[0].path, props.projects[0].kind);
  }
  pickAction.value = "";
  addOpen.value = true;
}

function confirmAddStep() {
  const project = pickedProject.value;
  if (!project || !pickAction.value) return;
  const id = newNodeId("step");
  const stepCount = nodes.value.filter((n) => n.type === NODE_STEP).length;
  const node = {
    id,
    type: NODE_STEP,
    position: { x: 240 + (stepCount % 3) * 40, y: 80 + stepCount * 70 },
    relPath: projectRelPath(project, project.rootPath || props.primaryRootPath),
    kind: project.kind,
    rootId: project.rootId || "",
    action: pickAction.value,
    wait: pickWait.value === "complete" ? "complete" : "spawn",
  };
  nodes.value = [...nodes.value, node];
  selectedId.value = id;
  addOpen.value = false;
}

function onEditStep(node) {
  const match = props.projects.find((p) => {
    const rel = projectRelPath(p, p.rootPath || props.primaryRootPath);
    return rel === node.relPath && p.kind === node.kind;
  });
  if (match) pickKey.value = makeProjectKey(match.path, match.kind);
  pickAction.value = node.action;
  pickWait.value = node.wait === "complete" ? "complete" : "spawn";
  selectedId.value = node.id;
  addOpen.value = true;
  // editing: on confirm replace
  editingStepId.value = node.id;
}

const editingStepId = ref("");

watch(addOpen, (v) => {
  if (!v) editingStepId.value = "";
});

function confirmAddOrEdit() {
  if (editingStepId.value) {
    const project = pickedProject.value;
    if (!project || !pickAction.value) return;
    nodes.value = nodes.value.map((n) =>
      n.id === editingStepId.value
        ? {
            ...n,
            relPath: projectRelPath(project, project.rootPath || props.primaryRootPath),
            kind: project.kind,
            rootId: project.rootId || "",
            action: pickAction.value,
            wait: pickWait.value === "complete" ? "complete" : "spawn",
          }
        : n,
    );
    addOpen.value = false;
    editingStepId.value = "";
    return;
  }
  confirmAddStep();
}

function onCanvasNodes(next) {
  nodes.value = next;
}

function onCanvasEdges(next) {
  edges.value = next;
}

function onCanvasSelected(id) {
  selectedId.value = id || "";
}

function deleteSelected() {
  const id = selectedId.value;
  if (!id) return;
  if (id.startsWith("edge:")) {
    const eid = id.slice(5);
    edges.value = edges.value.filter((e) => e.id !== eid);
    selectedId.value = "";
    return;
  }
  if (id === NODE_START || id === NODE_END) {
    message.warning(props.t("runPlanCannotDeleteTerminal"));
    return;
  }
  nodes.value = nodes.value.filter((n) => n.id !== id);
  edges.value = edges.value.filter((e) => e.source !== id && e.target !== id);
  selectedId.value = "";
}

function onDeletePlan(plan, ev) {
  ev?.stopPropagation?.();
  if (!plan?.id) return;
  Modal.confirm({
    title: props.t("runPlanDeleteConfirmTitle"),
    content: props.t("runPlanDeleteConfirmBody", { name: plan.name }),
    okText: props.t("runPlanDelete"),
    okType: "danger",
    cancelText: props.t("cancel"),
    onOk() {
      emit("delete", plan);
      if (planId.value === plan.id) resetForm();
    },
  });
}
</script>

<template>
  <div v-if="isPage" class="settings-layout plans-layout">
    <aside class="settings-nav">
      <div class="plans-nav-head">
        <div class="settings-nav-heading">{{ t("runPlanSavedList") }}</div>
        <button
          type="button"
          class="plans-nav-add"
          :class="{ active: !isEditingSaved }"
          :title="t('runPlanNew')"
          @click="onNewPlan"
        >
          <PlusOutlined />
        </button>
      </div>
      <div
        v-for="p in plans"
        :key="p.id"
        class="plan-nav-row"
        :class="{ active: planId === p.id }"
      >
        <button type="button" class="settings-nav-item plan-nav-item" @click="loadExisting(p)">
          <span class="plan-nav-name">{{ p.name }}</span>
          <span class="plan-nav-src">
            {{ p.source === "workspace" ? t("runPlanSourceWorkspace") : t("runPlanSourceUser") }}
          </span>
        </button>
        <button
          type="button"
          class="plan-nav-del"
          :title="t('runPlanDelete')"
          @click="onDeletePlan(p, $event)"
        >
          ×
        </button>
      </div>
      <p v-if="!plans.length" class="plans-nav-empty">{{ t("runPlanEmpty") }}</p>
    </aside>

    <main class="settings-main plan-main">
      <section class="plan-panel">
        <header class="plan-head">
          <div class="plan-head-text">
            <h2>{{ t("runPlanEditor") }}</h2>
            <p class="settings-desc">{{ t("runPlanPageLead") }}</p>
          </div>
          <div class="plan-head-fields">
            <a-input
              v-model:value="name"
              class="plan-name-input"
              :placeholder="t('runPlanNamePlaceholder')"
              allow-clear
            />
            <label class="plan-stop">
              <a-checkbox v-model:checked="stopOnError" />
              <span>{{ t("runPlanStopOnError") }}</span>
            </label>
            <a-button @click="openAddStep">{{ t("runPlanAddStepNode") }}</a-button>
            <a-button :disabled="!selectedId" @click="deleteSelected">
              {{ t("runPlanDeleteSelected") }}
            </a-button>
          </div>
        </header>

        <div class="plan-canvas-wrap">
          <PlanFlowCanvas
            :t="t"
            :nodes="nodes"
            :edges="edges"
            :selected-id="selectedId"
            @update:nodes="onCanvasNodes"
            @update:edges="onCanvasEdges"
            @update:selected-id="onCanvasSelected"
            @edit-step="onEditStep"
          />
        </div>

        <footer class="plan-footer">
          <div class="plan-summary" :class="{ ok: canSubmit }">{{ summaryText }}</div>
          <div class="plan-footer-actions">
            <a-button type="primary" :disabled="!canSubmit || isPlanRunning" @click="onExecute">
              {{ t("runPlanExecute") }}
            </a-button>
            <a-button danger :disabled="!isPlanRunning" @click="emit('stop')">
              {{ t("runPlanStop") }}
            </a-button>
            <a-button :disabled="!canSubmit" @click="onSaveUser">{{ t("runPlanSaveUser") }}</a-button>
            <a-button :disabled="!canSubmit" @click="onSaveWorkspace">
              {{ t("runPlanSaveWorkspace") }}
            </a-button>
            <a-button
              danger
              type="text"
              :disabled="!isEditingSaved || isPlanRunning"
              @click="
                onDeletePlan(
                  plans.find((p) => p.id === planId) || { id: planId, name: name || planId },
                )
              "
            >
              {{ t("runPlanDelete") }}
            </a-button>
          </div>
          <span class="plan-footer-hint">{{ t("runPlanFooterHint") }}</span>
        </footer>
      </section>
    </main>

    <a-modal
      v-model:open="addOpen"
      :title="editingStepId ? t('runPlanEditStep') : t('runPlanAddStepNode')"
      :ok-text="t('runPlanApplyStep')"
      :ok-button-props="{ disabled: !pickKey || !pickAction }"
      @ok="confirmAddOrEdit"
    >
      <div class="plan-add-form">
        <div class="cfg-label">{{ t("runPlanColProject") }}</div>
        <a-select
          v-model:value="pickKey"
          style="width: 100%"
          :options="projectOptions"
          show-search
          option-filter-prop="label"
        />
        <div class="cfg-label" style="margin-top: 10px">{{ t("runPlanColAction") }}</div>
        <a-select
          v-model:value="pickAction"
          style="width: 100%"
          :options="actionOptions"
          :placeholder="t('runPlanCmdPlaceholder')"
        />
        <div class="cfg-label" style="margin-top: 10px">{{ t("runPlanWaitCol") }}</div>
        <a-radio-group v-model:value="pickWait">
          <a-radio-button value="complete">{{ t("runPlanWaitCompleteShort") }}</a-radio-button>
          <a-radio-button value="spawn">{{ t("runPlanWaitSpawnShort") }}</a-radio-button>
        </a-radio-group>
      </div>
    </a-modal>
  </div>
</template>

<style scoped>
.plans-layout {
  height: 100%;
  min-height: 0;
  flex: 1;
}
.plans-nav-empty {
  margin: 8px;
  color: var(--muted);
  font-size: 11px;
  line-height: 1.4;
}
.plans-nav-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 6px;
  margin-bottom: 6px;
}
.plans-nav-head .settings-nav-heading {
  margin: 0;
  flex: 1;
  min-width: 0;
}
.plans-nav-add {
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border: 1px solid var(--line);
  border-radius: var(--radius, 3px);
  background: var(--panel, #fff);
  color: var(--ink-soft);
  font-size: 10px;
  cursor: pointer;
}
.plans-nav-add:hover,
.plans-nav-add.active {
  border-color: var(--teal);
  color: var(--teal);
  background: var(--teal-soft);
}
.plan-nav-row {
  display: flex;
  align-items: stretch;
  gap: 2px;
  margin-bottom: 2px;
  border-radius: var(--radius, 3px);
}
.plan-nav-row.active {
  background: var(--teal-soft);
}
.plan-nav-row .plan-nav-item {
  flex: 1;
  min-width: 0;
  margin-bottom: 0;
}
.plan-nav-row.active .plan-nav-item {
  background: transparent;
  color: var(--teal);
}
.plan-nav-item {
  display: flex;
  align-items: center;
  gap: 6px;
  justify-content: space-between;
}
.plan-nav-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.plan-nav-src {
  flex: none;
  font-size: 10px;
  font-weight: 600;
  color: var(--muted);
  border: 1px solid var(--line);
  border-radius: var(--radius, 3px);
  padding: 0 4px;
  line-height: 1.6;
}
.plan-nav-del {
  flex: none;
  width: 26px;
  border: 0;
  background: transparent;
  color: var(--muted);
  font-size: 16px;
  line-height: 1;
  border-radius: var(--radius, 3px);
  cursor: pointer;
}
.plan-nav-del:hover {
  background: color-mix(in srgb, #ef4444 12%, transparent);
  color: #dc2626;
}
.plan-main {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding-bottom: 0;
}
.plan-panel {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.plan-head-text h2 {
  margin: 0 0 2px;
  font-size: 15px;
  font-weight: 700;
}
.settings-desc {
  margin: 0 0 8px;
  color: var(--muted);
  font-size: 11px;
  line-height: 1.45;
}
.plan-head-fields {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.plan-name-input {
  width: min(240px, 100%);
}
.plan-stop {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin: 0;
  font-size: 12px;
  color: var(--ink-soft);
  cursor: pointer;
  white-space: nowrap;
}
.plan-canvas-wrap {
  flex: 1;
  min-height: 0;
}
.plan-footer {
  flex: none;
  display: grid;
  grid-template-columns: 1fr auto;
  grid-template-areas:
    "summary actions"
    "hint hint";
  gap: 6px 12px;
  align-items: center;
  padding: 8px 0 12px;
  border-top: 1px solid var(--line-soft);
}
.plan-summary {
  grid-area: summary;
  min-width: 0;
  padding: 6px 10px;
  border-radius: var(--radius, 3px);
  border: 1px solid var(--line);
  background: var(--surface-muted, #f5f7f6);
  color: var(--muted);
  font-size: 11px;
  line-height: 1.45;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.plan-summary.ok {
  border-color: color-mix(in srgb, var(--teal) 35%, var(--line));
  background: var(--teal-soft);
  color: var(--ink-soft);
  font-weight: 600;
}
.plan-footer-actions {
  grid-area: actions;
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  justify-content: flex-end;
}
.plan-footer-hint {
  grid-area: hint;
  font-size: 10px;
  color: var(--muted);
}
.cfg-label {
  font-size: 12px;
  font-weight: 600;
  margin-bottom: 4px;
}
</style>
