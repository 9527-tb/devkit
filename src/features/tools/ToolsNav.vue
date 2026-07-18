<!--
  工具页左侧导航：由 registry 驱动，新增工具无需改此文件逻辑。
-->
<script setup>
import { createTranslator } from "../../i18n/index.js";
import { locale } from "../../stores/settings.js";
import { toolsCat } from "../../stores/tools.js";
import { toolsSorted } from "./registry.js";

const t = createTranslator(locale);
const navItems = toolsSorted();
</script>

<template>
  <aside class="settings-nav">
    <button
      v-for="item in navItems"
      :key="item.id"
      type="button"
      class="settings-nav-item"
      :class="{ active: toolsCat === item.id }"
      @click="toolsCat = item.id"
    >
      {{ t(item.titleKey) }}
    </button>
  </aside>
</template>
