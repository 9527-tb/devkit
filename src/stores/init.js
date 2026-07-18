/**
 * 首次初始化弹窗状态：开关、步骤、进度。
 * 依赖：api/settings；被 features/init 消费。
 * 对应 DESIGN.md §12.2 stores/init
 */

import { ref } from "vue";
import { createTranslator } from "../i18n/index.js";
import { locale } from "./settings.js";

// TODO(fe-init-modal): DONE — bootstrap 流程与 init-progress 订阅迁入 features/init — DESIGN §6.1

export const INIT_STEP_DEFS = [
  { id: "prepare", labelKey: "initStepPrepare" },
  { id: "scanJdk", labelKey: "initStepJdk" },
  { id: "scanNode", labelKey: "initStepNode" },
  { id: "scanMaven", labelKey: "initStepMaven" },
  { id: "writeConfig", labelKey: "initStepWrite" },
  { id: "done", labelKey: "initStepDone" },
];

export const visible = ref(false);
export const inProgress = ref(false);
export const percent = ref(0);
export const step = ref("prepare");
export const detail = ref("");
export const done = ref(false);
export const stepStatus = ref(
  Object.fromEntries(INIT_STEP_DEFS.map((s) => [s.id, "pending"])),
);

const t = createTranslator(locale);

export function onOpenChange(open) {
  if (!open && inProgress.value && !done.value) {
    visible.value = true;
    return;
  }
  visible.value = open;
}

export function resetInitUi() {
  percent.value = 0;
  step.value = "prepare";
  detail.value = "";
  done.value = false;
  stepStatus.value = Object.fromEntries(
    INIT_STEP_DEFS.map((s) => [s.id, "pending"]),
  );
}

export function applyInitProgress(payload) {
  if (!payload || typeof payload !== "object") return;
  const stepId = payload.step || "prepare";
  const order = INIT_STEP_DEFS.map((s) => s.id);
  const idx = order.indexOf(stepId);
  const nextStatus = { ...stepStatus.value };
  order.forEach((id, i) => {
    if (i < idx) nextStatus[id] = "done";
    else if (i === idx) nextStatus[id] = payload.done ? "done" : "active";
    else if (nextStatus[id] !== "done") nextStatus[id] = "pending";
  });
  if (payload.done) {
    order.forEach((id) => {
      nextStatus[id] = "done";
    });
  }
  stepStatus.value = nextStatus;
  step.value = stepId;
  percent.value = Math.min(100, Math.max(0, Number(payload.percent) || 0));
  done.value = !!payload.done;

  if (stepId === "scanJdk" && payload.found != null) {
    detail.value = t("initFoundJdk", { n: payload.found });
  } else if (stepId === "scanNode" && payload.found != null) {
    detail.value = t("initFoundNode", { n: payload.found });
  } else if (stepId === "scanMaven") {
    detail.value =
      payload.found > 0 || payload.detail
        ? t("initFoundMaven")
        : t("initNoMaven");
  } else if (stepId === "writeConfig") {
    detail.value = t("initWriting");
  } else if (stepId === "done") {
    detail.value = t("initStepDone");
  } else if (stepId === "prepare") {
    detail.value = t("initStepPrepare");
  }
}
