<!--
  顶栏：品牌 | 路径 Compact+历史+分析 | 工具图标 | 设置图标。
  对应 DESIGN.md §12.2 Topbar
-->
<script setup>
import { computed, inject } from "vue";
import {
  FolderOpenOutlined,
  ReloadOutlined,
  SettingOutlined,
  EllipsisOutlined,
  ToolOutlined,
} from "@antdv-next/icons";
// DONE(fe-topbar-history): Compact + 历史 Dropdown — DESIGN §12.2

const appLogo = "/logo.png";
const appUpdate = inject("appUpdate", null);
const updateAvailable = computed(() => !!appUpdate?.updateAvailable?.value);

defineProps({
  t: { type: Function, required: true },
  root: { type: String, default: "" },
  loading: { type: Boolean, default: false },
  previewMode: { type: Boolean, default: false },
  workspaceHistoryMenu: { type: Object, required: true },
  dropdownPopupContainer: { type: Function, required: true },
});

defineEmits(["choose-directory", "scan", "open-settings", "open-tools"]);
</script>

<template>
  <header class="topbar">
    <div class="brand">
      <img class="brand-mark" :src="appLogo" alt="" />
      <div class="brand-text">
        <strong>{{ t("appName") }}</strong>
        <small>{{ t("appTagline") }}</small>
      </div>
    </div>

    <div class="workspace-picker">
      <div class="path-chip" :title="root || undefined">
        <FolderOpenOutlined class="path-chip-icon" />
        <span>{{ root || t("selectProjectHint") }}</span>
      </div>
      <a-space-compact>
        <a-button type="primary" @click="$emit('choose-directory')">
          <template #icon><FolderOpenOutlined /></template>
          <span class="label">{{ t("chooseProject") }}</span>
        </a-button>
        <a-dropdown
          :menu="workspaceHistoryMenu"
          placement="bottomRight"
          :trigger="['click']"
          :get-popup-container="dropdownPopupContainer"
          overlay-class-name="workspace-history-dropdown"
          :overlay-style="{ zIndex: 3100 }"
        >
          <a-button type="primary" :title="t('workspaceHistory')">
            <template #icon><EllipsisOutlined /></template>
          </a-button>
        </a-dropdown>
      </a-space-compact>
      <a-button :loading="loading" @click="$emit('scan')">
        <template #icon><ReloadOutlined /></template>
        <span class="label">{{ t("analyze") }}</span>
      </a-button>
    </div>

    <div class="top-actions">
      <a-tag v-if="previewMode" color="processing">{{ t("previewMode") }}</a-tag>
      <a-button
        class="settings-icon-btn"
        :title="t('tools')"
        @click="$emit('open-tools')"
      >
        <template #icon><ToolOutlined /></template>
      </a-button>
      <a-badge :dot="updateAvailable" :offset="[-4, 4]">
        <a-button
          class="settings-icon-btn"
          :title="t('settings')"
          @click="$emit('open-settings')"
        >
          <template #icon><SettingOutlined /></template>
        </a-button>
      </a-badge>
    </div>
  </header>
</template>
