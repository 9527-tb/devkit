<!--
  设置页视图壳：左导航 + 右侧动态面板。
  依赖：features/settings；禁止写死仅 java/node 两页复制逻辑。
  对应 DESIGN.md §12.2 SettingsView
-->
<script setup>
import { computed } from "vue";
import { LeftOutlined } from "@antdv-next/icons";
import { createTranslator } from "../i18n/index.js";
import { locale, settingsLoading, settingsCat } from "../stores/settings.js";
import { useSettings } from "../features/settings/useSettings.js";
import SettingsNav from "../features/settings/SettingsNav.vue";
import SettingsGeneral from "../features/settings/SettingsGeneral.vue";
import SettingsToolchain from "../features/settings/SettingsToolchain.vue";

// TODO(fe-settings-view): DONE — 从 App.vue 迁出设置页布局与导航切换 — DESIGN §12.2

const appLogo = "/logo.png";
const t = createTranslator(locale);
const { closeSettings } = useSettings();

const showGeneral = computed(() => settingsCat.value === "general");
</script>

<template>
  <a-config-provider component-size="small">
    <header class="topbar">
      <div class="brand">
        <img class="brand-mark" :src="appLogo" alt="" />
        <div class="brand-text">
          <span>{{ t("settings") }}</span>
          <small>{{ t("settingsFile") }}</small>
        </div>
      </div>
      <div class="settings-top-actions">
        <a-tag color="orange" class="settings-auto-tag">{{ t("autoSave") }}</a-tag>
        <a-button @click="closeSettings">
          <template #icon><LeftOutlined /></template>
          {{ t("back") }}
        </a-button>
      </div>
    </header>

    <div class="settings-layout">
      <SettingsNav />

      <a-spin :spinning="settingsLoading" class="settings-main-spin">
        <main class="settings-main">
          <SettingsGeneral v-show="showGeneral" />
          <SettingsToolchain />
        </main>
      </a-spin>
    </div>
  </a-config-provider>
</template>

<style>
@import "../styles/settings-layout.css";

.theme-grid {
  display: grid;
  grid-template-columns: repeat(6, minmax(0, 1fr));
  gap: 8px;
}
@media (max-width: 900px) {
  .theme-grid { grid-template-columns: repeat(3, minmax(0, 1fr)); }
}
/* macOS WKWebView：button 内空 span + 内联 background 常不绘色，改用 class + background-color */
.theme-swatch {
  -webkit-appearance: none;
  appearance: none;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px;
  border-radius: var(--radius);
  border: 1px solid var(--line);
  background-color: var(--surface-muted);
  color: inherit;
  font: inherit;
  cursor: pointer;
  text-align: left;
}
.theme-swatch:hover { border-color: var(--line-hover); }
.theme-swatch.active {
  border-color: var(--teal);
  box-shadow: 0 0 0 1px var(--teal);
  background-color: var(--teal-soft);
}
.theme-swatch-bar {
  display: block;
  width: 100%;
  height: 22px;
  min-height: 22px;
  flex-shrink: 0;
  border-radius: var(--radius);
  background-color: #999;
}
/* 与 themes/index.js THEME_OPTIONS[].primary 保持一致 */
.theme-swatch--teal .theme-swatch-bar { background-color: #0f766e; }
.theme-swatch--ocean .theme-swatch-bar { background-color: #0c6a8a; }
.theme-swatch--forest .theme-swatch-bar { background-color: #3d6b4f; }
.theme-swatch--slate .theme-swatch-bar { background-color: #4a5d6a; }
.theme-swatch--amber .theme-swatch-bar { background-color: #b45309; }
.theme-swatch--rose .theme-swatch-bar { background-color: #9f1239; }
.theme-swatch-name {
  font-size: 11px;
  font-weight: 700;
  color: var(--ink-soft);
}
.locale-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
.locale-chip {
  height: 30px;
  padding: 0 12px;
  border-radius: var(--radius);
  border: 1px solid var(--line);
  background: var(--surface-muted);
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-soft);
  cursor: pointer;
}
.locale-chip:hover { border-color: var(--line-hover); }
.locale-chip.active {
  border-color: var(--teal);
  background: var(--teal-soft);
  color: var(--teal);
}
.runtime-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.runtime-list li {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: var(--radius);
  background: var(--panel);
  border: 1px solid var(--line);
}
.runtime-list li > div {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.runtime-list strong {
  font-size: 12px;
  color: var(--ink-strong);
}
.runtime-list small {
  color: var(--muted);
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}
.runtime-add {
  display: flex;
  gap: 6px;
}
.runtime-add .ant-input {
  flex: 1;
  min-width: 0;
}
</style>
