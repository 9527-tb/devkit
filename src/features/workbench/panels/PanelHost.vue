<!--
  动态面板宿主：按 panelId 渲染注册表组件。
  对应 DESIGN.md §8 / §12.2 PanelHost
-->
<script setup>
import { computed } from "vue";
import { createTranslator } from "../../../i18n/index.js";
import { locale } from "../../../stores/settings.js";
import { resolvePanel } from "./registry.js";

// DONE(fe-panel-registry): panelId → 组件 + PanelHost — DESIGN §8.1

const props = defineProps({
  panelId: { type: String, required: true },
  panelProps: { type: Object, default: () => ({}) },
});

const t = createTranslator(locale);
const component = computed(() => resolvePanel(props.panelId));
</script>

<template>
  <component :is="component" v-if="component" v-bind="panelProps">
    <slot />
  </component>
  <div v-else class="panel-missing">{{ t("unknownPanel", { id: panelId }) }}</div>
</template>

<style scoped>
.panel-missing {
  padding: 16px;
  color: var(--dk-muted, #6b7a76);
}
</style>
