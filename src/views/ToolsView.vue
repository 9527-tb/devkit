<!--
  工具页壳：左导航（registry）+ 右侧动态工具面板。
-->
<script setup>
import { computed } from "vue";
import { LeftOutlined } from "@antdv-next/icons";
import { createTranslator } from "../i18n/index.js";
import { locale } from "../stores/settings.js";
import { toolsCat } from "../stores/tools.js";
import { useTools } from "../features/tools/useTools.js";
import ToolsNav from "../features/tools/ToolsNav.vue";
import { findTool } from "../features/tools/registry.js";
import "../styles/settings-layout.css";

const appLogo = "/logo.png";
const t = createTranslator(locale);
const { closeTools } = useTools();

const activeTool = computed(() => findTool(toolsCat.value));
</script>

<template>
  <a-config-provider component-size="small">
    <header class="topbar">
      <div class="brand">
        <img class="brand-mark" :src="appLogo" alt="" />
        <div class="brand-text">
          <span>{{ t("tools") }}</span>
          <small>{{ t("toolsSubtitle") }}</small>
        </div>
      </div>
      <div class="settings-top-actions">
        <a-button @click="closeTools">
          <template #icon><LeftOutlined /></template>
          {{ t("back") }}
        </a-button>
      </div>
    </header>

    <div class="settings-layout">
      <ToolsNav />
      <main class="settings-main">
        <component :is="activeTool.component" v-if="activeTool" :key="activeTool.id" />
        <section v-else class="settings-panel">
          <p class="settings-desc">{{ t("toolsEmpty") }}</p>
        </section>
      </main>
    </div>
  </a-config-provider>
</template>
