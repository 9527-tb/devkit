<!--
  设置页左侧导航：通用段 + 动态 toolchain 段。
  依赖：stores/settings；被 SettingsView 使用。
  对应 DESIGN.md §12.2 SettingsNav
-->
<script setup>
import { computed, inject, onMounted, ref } from "vue";
import { createTranslator } from "../../i18n/index.js";
import { locale, settingsCat } from "../../stores/settings.js";
import { invokeSafe } from "../../api/tauri.js";

// DONE(fe-settings-view): 导航项与选中态 — DESIGN §12.2
// DONE(fe-settings-toolchain-dynamic): 按 list_providers 动态段 — DESIGN R5

const t = createTranslator(locale);
const appUpdate = inject("appUpdate", null);
const updateAvailable = computed(() => !!appUpdate?.updateAvailable?.value);

const staticItems = [{ id: "general", labelKey: "navGeneral" }];

/** @type {import('vue').Ref<Array<{id:string,labelKey:string}>>} */
const toolchainItems = ref([
  { id: "java", labelKey: "navJava" },
  { id: "node", labelKey: "navNode" },
]);

const KIND_NAV = {
  maven: { id: "java", labelKey: "navJava" },
  node: { id: "node", labelKey: "navNode" },
  cargo: { id: "cargo", labelKey: "navCargo" },
  gradle: { id: "gradle", labelKey: "navGradle" },
};

onMounted(async () => {
  try {
    const kinds = await invokeSafe("list_providers");
    if (!Array.isArray(kinds) || !kinds.length) return;
    const seen = new Set();
    const items = [];
    for (const kind of kinds) {
      const meta = KIND_NAV[kind];
      if (!meta || seen.has(meta.id)) continue;
      seen.add(meta.id);
      items.push(meta);
    }
    if (items.length) toolchainItems.value = items;
  } catch {
    /* preview */
  }
});

const navItems = computed(() => [...staticItems, ...toolchainItems.value]);
</script>

<template>
  <aside class="settings-nav">
    <button
      v-for="item in navItems"
      :key="item.id"
      type="button"
      class="settings-nav-item"
      :class="{ active: settingsCat === item.id }"
      @click="settingsCat = item.id"
    >
      <a-badge
        :dot="item.id === 'general' && updateAvailable"
        :offset="[6, 2]"
      >
        <span>{{ t(item.labelKey) }}</span>
      </a-badge>
    </button>
  </aside>
</template>
