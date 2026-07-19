<!--
  设置页左侧导航：基本配置 + 工具链段 + 项目类型段。
  依赖：stores/settings；被 SettingsView 使用。
  对应 DESIGN.md §12.2 SettingsNav
-->
<script setup>
import { computed, inject, onMounted, ref, watch } from "vue";
import { createTranslator } from "../../i18n/index.js";
import {
  locale,
  settingsCat,
  normalizeSettingsCat,
  SETTINGS_CAT,
  PROVIDER_KIND_CAT,
} from "../../stores/settings.js";
import { invokeSafe } from "../../api/tauri.js";

const t = createTranslator(locale);
const appUpdate = inject("appUpdate", null);
const updateAvailable = computed(() => !!appUpdate?.updateAvailable?.value);

/** 项目类型侧栏固定顺序：Node → Cargo → Maven → Gradle */
const PROVIDER_NAV_ORDER = ["node", "cargo", "maven", "gradle"];

const PROVIDER_LABEL_KEYS = {
  node: "navProviderNode",
  cargo: "navProviderCargo",
  maven: "navProviderMaven",
  gradle: "navProviderGradle",
};

/** @type {import('vue').Ref<string[]>} */
const providerKinds = ref([...PROVIDER_NAV_ORDER]);

onMounted(async () => {
  settingsCat.value = normalizeSettingsCat(settingsCat.value);
  try {
    const kinds = await invokeSafe("list_providers");
    if (!Array.isArray(kinds) || !kinds.length) return;
    const available = new Set(kinds.filter((k) => PROVIDER_KIND_CAT[k]));
    const ordered = PROVIDER_NAV_ORDER.filter((k) => available.has(k));
    if (ordered.length) providerKinds.value = ordered;
  } catch {
    /* preview */
  }
});

watch(settingsCat, (cat) => {
  const next = normalizeSettingsCat(cat);
  if (next !== cat) settingsCat.value = next;
});

/** @type {import('vue').ComputedRef<Array<{type:'heading'|'link', id?:string, labelKey:string, nested?:boolean}>>} */
const navItems = computed(() => {
  const providers = providerKinds.value.map((kind) => ({
    type: "link",
    id: PROVIDER_KIND_CAT[kind],
    labelKey: PROVIDER_LABEL_KEYS[kind],
    nested: true,
  }));
  return [
    { type: "link", id: SETTINGS_CAT.GENERAL, labelKey: "navGeneral" },
    { type: "heading", labelKey: "navSectionToolchain" },
    {
      type: "link",
      id: SETTINGS_CAT.TOOLCHAIN_JDK,
      labelKey: "navToolchainJava",
      nested: true,
    },
    {
      type: "link",
      id: SETTINGS_CAT.TOOLCHAIN_NODE,
      labelKey: "navToolchainNode",
      nested: true,
    },
    { type: "heading", labelKey: "navSectionProviders" },
    ...providers,
  ];
});

function selectCat(id) {
  settingsCat.value = normalizeSettingsCat(id);
}
</script>

<template>
  <aside class="settings-nav">
    <template v-for="(item, index) in navItems" :key="item.id || `${item.labelKey}-${index}`">
      <div v-if="item.type === 'heading'" class="settings-nav-heading">
        {{ t(item.labelKey) }}
      </div>
      <button
        v-else
        type="button"
        class="settings-nav-item"
        :class="{ active: settingsCat === item.id, nested: item.nested }"
        @click="selectCat(item.id)"
      >
        <a-badge
          :dot="item.id === SETTINGS_CAT.GENERAL && updateAvailable"
          :offset="[6, 2]"
        >
          <span>{{ t(item.labelKey) }}</span>
        </a-badge>
      </button>
    </template>
  </aside>
</template>
