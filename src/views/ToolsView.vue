<!--
  工具页壳：左导航（registry）+ 右侧动态工具面板。
-->
<script setup>
import { computed } from "vue";
import { toolsCat } from "../stores/tools.js";
import ToolsNav from "../features/tools/ToolsNav.vue";
import { findTool } from "../features/tools/registry.js";
import { createTranslator } from "../i18n/index.js";
import { locale } from "../stores/settings.js";
import "../styles/settings-layout.css";

const t = createTranslator(locale);
const activeTool = computed(() => findTool(toolsCat.value));
</script>

<template>
  <a-config-provider component-size="small">
    <div class="tools-page">
      <div class="settings-layout">
        <ToolsNav />
        <main class="settings-main">
          <component :is="activeTool.component" v-if="activeTool" :key="activeTool.id" />
          <section v-else class="settings-panel">
            <p class="settings-desc">{{ t("toolsEmpty") }}</p>
          </section>
        </main>
      </div>
    </div>
  </a-config-provider>
</template>

<style scoped>
.tools-page {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>
