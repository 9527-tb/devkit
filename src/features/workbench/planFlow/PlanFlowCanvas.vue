<!--
  Vue Flow 编排画布。
  位置由 Vue Flow v-model 管理；仅在拖拽结束/结构变更时与父级 plan 模型同步。
-->
<script setup>
import { markRaw, nextTick, ref, watch } from "vue";
import { VueFlow } from "@vue-flow/core";
import { Background } from "@vue-flow/background";
import "@vue-flow/core/dist/style.css";
import "@vue-flow/core/dist/theme-default.css";
import { NODE_END, NODE_START, NODE_STEP } from "../runPlans.js";
import PlanStartNode from "./PlanStartNode.vue";
import PlanEndNode from "./PlanEndNode.vue";
import PlanStepNode from "./PlanStepNode.vue";

const props = defineProps({
  t: { type: Function, required: true },
  nodes: { type: Array, default: () => [] },
  edges: { type: Array, default: () => [] },
  selectedId: { type: String, default: "" },
});

const emit = defineEmits([
  "update:nodes",
  "update:edges",
  "update:selectedId",
  "edit-step",
]);

const flowRef = ref(null);
/** Vue Flow 节点（含实时拖拽坐标） */
const localNodes = ref([]);
const localEdges = ref([]);

const nodeTypes = {
  start: markRaw(PlanStartNode),
  end: markRaw(PlanEndNode),
  step: markRaw(PlanStepNode),
};

function planNodeToVf(node) {
  const base = {
    id: node.id,
    type: node.type,
    position: {
      x: Number(node.position?.x) || 0,
      y: Number(node.position?.y) || 0,
    },
    deletable: node.type === NODE_STEP,
    selectable: true,
  };
  if (node.type === NODE_START) {
    return { ...base, data: { label: props.t("runPlanNodeStart") } };
  }
  if (node.type === NODE_END) {
    return { ...base, data: { label: props.t("runPlanNodeEnd") } };
  }
  const id = node.id;
  return {
    ...base,
    data: {
      kind: node.kind,
      relPath: node.relPath,
      rootId: node.rootId || "",
      action: node.action,
      wait: node.wait,
      waitCompleteLabel: props.t("runPlanWaitCompleteShort"),
      waitSpawnLabel: props.t("runPlanWaitSpawnShort"),
      onEdit: () => {
        const latest = props.nodes.find((n) => n.id === id);
        if (latest) emit("edit-step", latest);
      },
    },
  };
}

function vfNodeToPlan(node) {
  const position = {
    x: Number(node.position?.x) || 0,
    y: Number(node.position?.y) || 0,
  };
  if (node.type === NODE_START || node.type === NODE_END) {
    return { id: node.id, type: node.type, position };
  }
  const d = node.data || {};
  return {
    id: node.id,
    type: NODE_STEP,
    position,
    relPath: d.relPath || "",
    kind: d.kind || "",
    rootId: d.rootId || "",
    action: d.action || "",
    wait: d.wait === "complete" ? "complete" : "spawn",
  };
}

function planEdgeToVf(e) {
  return {
    id: e.id,
    source: e.source,
    target: e.target,
    sourceHandle: "out",
    targetHandle: "in",
    selectable: true,
    style: { stroke: "var(--teal, #0f766e)", strokeWidth: 2 },
  };
}

/** 结构签名（不含 position / selected） */
function structureKey(nodes, edges) {
  const n = (nodes || [])
    .map((x) =>
      [
        x.id,
        x.type,
        x.relPath || x.data?.relPath || "",
        x.kind || x.data?.kind || "",
        x.action || x.data?.action || "",
        x.wait || x.data?.wait || "",
        x.rootId || x.data?.rootId || "",
      ].join(":"),
    )
    .join("|");
  const e = (edges || []).map((x) => `${x.id}:${x.source}->${x.target}`).join("|");
  return `${n}#${e}`;
}

function syncNodesToParent() {
  emit(
    "update:nodes",
    localNodes.value.map(vfNodeToPlan),
  );
}

function syncEdgesToParent() {
  emit(
    "update:edges",
    localEdges.value.map((e) => ({
      id: e.id,
      source: e.source,
      target: e.target,
    })),
  );
}

/**
 * 从父级重建。
 * keepPositions：同一编辑会话内保留 local 坐标（避免用父级旧坐标覆盖）。
 */
function hydrateFromProps({ fit = false, keepPositions = false } = {}) {
  const posById = keepPositions
    ? new Map(
        localNodes.value.map((n) => [
          n.id,
          { x: Number(n.position?.x) || 0, y: Number(n.position?.y) || 0 },
        ]),
      )
    : null;

  localNodes.value = (props.nodes || []).map((n) => {
    const vf = planNodeToVf(n);
    if (posById?.has(n.id)) vf.position = { ...posById.get(n.id) };
    return vf;
  });

  localEdges.value = (props.edges || []).map(planEdgeToVf);

  if (fit) {
    nextTick(() => {
      try {
        flowRef.value?.fitView?.({ padding: 0.24, duration: 180 });
      } catch {
        /* ignore */
      }
    });
  }
}

watch(
  () => structureKey(props.nodes, props.edges),
  (next, prev) => {
    if (!prev) {
      hydrateFromProps({ fit: true, keepPositions: false });
      return;
    }
    if (next === prev) return;

    // 先前步骤仍在新图中 → 同会话增删改，保留已拖坐标；换计划则用父级坐标
    const prevIds = new Set(
      (prev.split("#")[0] || "")
        .split("|")
        .filter(Boolean)
        .map((s) => s.split(":")[0]),
    );
    const nextIds = new Set(
      (next.split("#")[0] || "")
        .split("|")
        .filter(Boolean)
        .map((s) => s.split(":")[0]),
    );
    const stepPrev = [...prevIds].filter((id) => id !== NODE_START && id !== NODE_END);
    const overlap = stepPrev.filter((id) => nextIds.has(id));
    // 换计划（步骤完全无交集）才丢弃本地坐标；增删改步骤保留已拖位置
    const isPlanReplace = stepPrev.length > 0 && overlap.length === 0;

    hydrateFromProps({ fit: false, keepPositions: !isPlanReplace });
    nextTick(() => {
      syncNodesToParent();
      syncEdgesToParent();
    });
  },
  { immediate: true },
);

function nodeTypeOf(id) {
  return localNodes.value.find((n) => n.id === id)?.type || "";
}

function isValidConnection(conn) {
  if (!conn?.source || !conn?.target) return false;
  if (conn.source === conn.target) return false;
  if (nodeTypeOf(conn.source) === NODE_END) return false;
  if (nodeTypeOf(conn.target) === NODE_START) return false;
  return true;
}

function onConnect(conn) {
  if (!isValidConnection(conn)) return;
  if (localEdges.value.some((e) => e.source === conn.source && e.target === conn.target)) return;
  localEdges.value = [
    ...localEdges.value,
    planEdgeToVf({
      id: `e_${conn.source}_${conn.target}_${Date.now().toString(36)}`,
      source: conn.source,
      target: conn.target,
    }),
  ];
  syncEdgesToParent();
}

function onNodeDragStop() {
  // v-model 已更新 localNodes；此处写回父级，供保存/校验
  syncNodesToParent();
}

function onSelectionChange({ nodes: selNodes, edges: selEdges }) {
  if (selNodes?.length) {
    emit("update:selectedId", selNodes[0].id);
    return;
  }
  if (selEdges?.length) {
    emit("update:selectedId", `edge:${selEdges[0].id}`);
    return;
  }
  if (props.selectedId) emit("update:selectedId", "");
}

function onNodesChange(changes) {
  // 位置由 v-model 维护；这里只在删除后同步父级
  if (!changes.some((ch) => ch.type === "remove")) return;
  nextTick(() => {
    const ids = new Set(localNodes.value.map((n) => n.id));
    localEdges.value = localEdges.value.filter((e) => ids.has(e.source) && ids.has(e.target));
    syncNodesToParent();
    syncEdgesToParent();
    if (props.selectedId && !ids.has(props.selectedId)) emit("update:selectedId", "");
  });
}

function onEdgesChange(changes) {
  if (!changes.some((ch) => ch.type === "remove")) return;
  syncEdgesToParent();
  const edgeSel = props.selectedId.startsWith("edge:") ? props.selectedId.slice(5) : "";
  if (edgeSel && changes.some((ch) => ch.type === "remove" && ch.id === edgeSel)) {
    emit("update:selectedId", "");
  }
}
</script>

<template>
  <div class="pf-wrap">
    <VueFlow
      ref="flowRef"
      v-model:nodes="localNodes"
      v-model:edges="localEdges"
      class="pf-flow"
      :node-types="nodeTypes"
      :nodes-draggable="true"
      :nodes-connectable="true"
      :elements-selectable="true"
      :min-zoom="0.4"
      :max-zoom="1.6"
      :is-valid-connection="isValidConnection"
      fit-view-on-init
      elevate-edges-on-select
      :delete-key-code="['Backspace', 'Delete']"
      @connect="onConnect"
      @node-drag-stop="onNodeDragStop"
      @selection-change="onSelectionChange"
      @nodes-change="onNodesChange"
      @edges-change="onEdgesChange"
    >
      <Background :gap="16" :size="1" color="var(--line)" />
    </VueFlow>
    <div class="pf-tip">{{ t("runPlanFlowTip") }}</div>
  </div>
</template>

<style scoped>
.pf-wrap {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 360px;
  border: 1px solid var(--line);
  border-radius: var(--radius, 3px);
  overflow: hidden;
  background: color-mix(in srgb, var(--panel) 88%, var(--surface-muted, #f5f7f6));
}
.pf-flow {
  width: 100%;
  height: 100%;
}
.pf-tip {
  position: absolute;
  left: 8px;
  bottom: 8px;
  z-index: 5;
  padding: 4px 8px;
  border-radius: var(--radius, 3px);
  background: color-mix(in srgb, var(--panel) 92%, transparent);
  border: 1px solid var(--line);
  font-size: 10px;
  color: var(--muted);
  pointer-events: none;
}
.pf-wrap :deep(.vue-flow__edge.selected .vue-flow__edge-path) {
  stroke: #d97706 !important;
  stroke-width: 2.5 !important;
}
.pf-wrap :deep(.vue-flow__connection-path) {
  stroke: var(--teal);
}
</style>
