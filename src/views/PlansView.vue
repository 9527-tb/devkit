<script setup>
import { computed, inject } from "vue";
import RunPlanEditor from "../features/workbench/RunPlanEditor.vue";
import { root } from "../stores/workspace.js";
import "../styles/settings-layout.css";

defineProps({
  t: { type: Function, required: true },
});

const workbench = inject("workbench");
const projects = computed(() => workbench?.projects?.value || []);
const mergedRunPlans = computed(() => workbench?.mergedRunPlans?.value || []);
const runningPlan = computed(() => workbench?.runPlanActive?.value || null);
</script>

<template>
  <a-config-provider component-size="small">
    <div class="plans-page">
      <RunPlanEditor
        layout="page"
        :open="true"
        :t="t"
        :projects="projects"
        :plans="mergedRunPlans"
        :primary-root-path="root"
        :running-plan="runningPlan"
        @execute="(plan) => workbench.executeRunPlan(plan)"
        @stop="() => workbench.stopRunPlan()"
        @delete="(plan) => workbench.deleteRunPlan(plan)"
        @save-user="(plan) => workbench.persistRunPlanUser(plan)"
        @save-workspace="(plan) => workbench.persistRunPlanWorkspace(plan)"
      />
    </div>
  </a-config-provider>
</template>

<style scoped>
.plans-page {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.plans-page :deep(.plans-layout) {
  flex: 1;
  min-height: 0;
}
</style>
