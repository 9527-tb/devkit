<!--
  部署面板占位（无真实远程副作用）。
  对应 DESIGN.md §9 / panel.deploy
-->
<script setup>
import { createTranslator } from "../../i18n/index.js";
import { locale } from "../../stores/settings.js";

// DONE(deploy-panel-stub): panel.deploy 占位组件 — DESIGN §9

defineProps({
  project: { type: Object, default: null },
});

const t = createTranslator(locale);
</script>

<template>
  <div class="deploy-panel">
    <h3>{{ t("deployTitle") }}</h3>
    <p class="muted">{{ t("deployDesc") }}</p>
    <a-empty :description="t('deployEmpty')" />
  </div>
</template>

<style scoped>
.deploy-panel {
  padding: 16px;
}
.deploy-panel h3 {
  margin: 0 0 8px;
  font-size: 14px;
}
.muted {
  color: var(--dk-muted, #6b7a76);
  font-size: 12px;
  margin-bottom: 16px;
}
</style>
