<!--
  设置「基本配置」面板：日志换行、主题、语言等。
  依赖：stores/settings、themes、i18n、useSettings。
  对应 DESIGN.md §12.2 SettingsGeneral
-->
<script setup>
import { inject, computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { message } from "antdv-next";
import { THEME_OPTIONS, THEME_LABEL_KEYS } from "../../themes/index.js";
import { LOCALE_OPTIONS, createTranslator } from "../../i18n/index.js";
import { locale, settings, previewMode } from "../../stores/settings.js";
import { workspaceConfig, lastScannedRoot } from "../../stores/workspace.js";
import { useSettings } from "./useSettings.js";

// TODO(fe-settings-general): DONE — 从 App.vue 迁出基本配置表单 — DESIGN §12.2

const t = createTranslator(locale);
const {
  onThemeChange,
  onLocaleChange,
  onLogWrapSettingChange,
  onNotifyOnTaskDoneChange,
  onHealthCheckOnScanChange,
  onPreferWorkspaceConfigChange,
  onEditorCommandChange,
  onTerminalAppChange,
  onActionButtonCountChange,
  onLaunchAtLoginChange,
  onCloseToTrayChange,
  onAutoCheckUpdateChange,
} = useSettings();

const externalTools = ref({
  editors: [],
  terminals: [],
  defaultEditor: "",
  defaultTerminal: "",
});

const editorSelectValue = computed(() => {
  const cur = settings.value.general?.editorCommand || "";
  return cur || externalTools.value.defaultEditor || undefined;
});

const terminalSelectValue = computed(() => {
  const cur = settings.value.general?.terminalApp || "";
  return cur || externalTools.value.defaultTerminal || undefined;
});

const editorOptions = computed(() => {
  const list = [...(externalTools.value.editors || [])];
  const cur = settings.value.general?.editorCommand;
  if (cur && !list.some((o) => o.id === cur)) {
    list.push({ id: cur, label: cur });
  }
  return list.map((o) => ({ value: o.id, label: o.label }));
});

const terminalOptions = computed(() => {
  const list = [...(externalTools.value.terminals || [])];
  const cur = settings.value.general?.terminalApp;
  if (cur && !list.some((o) => o.id === cur)) {
    list.push({ id: cur, label: cur });
  }
  return list.map((o) => ({ value: o.id, label: o.label }));
});

async function loadExternalTools() {
  if (previewMode.value) {
    externalTools.value = {
      editors: [
        { id: "cursor", label: "Cursor" },
        { id: "code", label: "VS Code" },
      ],
      terminals: [{ id: "terminal", label: "Terminal" }],
      defaultEditor: "cursor",
      defaultTerminal: "terminal",
    };
    return;
  }
  try {
    externalTools.value = await invoke("detect_external_tools");
    // 首次为空时写入探测到的系统默认，便于设置页直接展示
    const g = settings.value.general || {};
    if (!g.editorCommand && externalTools.value.defaultEditor) {
      onEditorCommandChange(externalTools.value.defaultEditor);
    }
    if (!g.terminalApp && externalTools.value.defaultTerminal) {
      onTerminalAppChange(externalTools.value.defaultTerminal);
    }
  } catch (error) {
    console.error(error);
  }
}

onMounted(() => {
  loadExternalTools();
});

const appUpdate = inject("appUpdate", null);
const updateChecking = computed(() => !!appUpdate?.checking?.value);
const updateAvailable = computed(() => !!appUpdate?.updateAvailable?.value);
const appVersionLabel = computed(() => appUpdate?.appVersion?.value || "—");

function onCheckUpdateClick() {
  appUpdate?.checkForUpdate?.({ silent: false });
}

async function onExportWorkspaceConfig() {
  const root = lastScannedRoot.value;
  if (!root || previewMode.value) {
    message.warning(t("workspaceConfigNeedScan"));
    return;
  }
  try {
    const config = {
      version: 1,
      projectFilter: settings.value.projectFilter,
      pipelines: workspaceConfig.value?.pipelines || [],
      probes: workspaceConfig.value?.probes || {},
    };
    await invoke("save_workspace_config", { root, config });
    workspaceConfig.value = config;
    message.success(t("workspaceConfigExported"));
  } catch (error) {
    message.error(String(error));
  }
}
</script>

<template>
  <section class="settings-panel">
    <h2>{{ t("generalTitle") }}</h2>
    <p class="settings-desc">{{ t("generalDesc") }}</p>

    <div class="cfg-group">
      <div class="cfg-group-h">
        <div>
          <b>{{ t("groupTheme") }}</b>
          <span>{{ t("groupThemeSub") }}</span>
        </div>
      </div>
      <div class="cfg-group-b">
        <div class="theme-grid">
          <button
            v-for="item in THEME_OPTIONS"
            :key="item.id"
            type="button"
            class="theme-swatch"
            :class="[`theme-swatch--${item.id}`, { active: settings.general.theme === item.id }]"
            @click="onThemeChange(item.id)"
          >
            <span class="theme-swatch-bar" aria-hidden="true" />
            <span class="theme-swatch-name">{{ t(THEME_LABEL_KEYS[item.id]) }}</span>
          </button>
        </div>
      </div>
    </div>

    <div class="cfg-group">
      <div class="cfg-group-h">
        <div>
          <b>{{ t("groupLocale") }}</b>
          <span>{{ t("groupLocaleSub") }}</span>
        </div>
      </div>
      <div class="cfg-group-b">
        <div class="locale-grid">
          <button
            v-for="item in LOCALE_OPTIONS"
            :key="item.id"
            type="button"
            class="locale-chip"
            :class="{ active: settings.general.locale === item.id }"
            @click="onLocaleChange(item.id)"
          >
            {{ item.label }}
          </button>
        </div>
      </div>
    </div>

    <div class="cfg-group">
      <div class="cfg-group-h">
        <div>
          <b>{{ t("groupLog") }}</b>
          <span>{{ t("groupLogSub") }}</span>
        </div>
      </div>
      <div class="cfg-group-b">
        <div class="cfg-row">
          <div>
            <div class="cfg-label">{{ t("logWrap") }}</div>
            <div class="cfg-hint">{{ t("logWrapHint") }}</div>
          </div>
          <a-switch
            :checked="settings.general.logWrap"
            @change="onLogWrapSettingChange"
          />
        </div>
      </div>
    </div>

    <div class="cfg-group">
      <div class="cfg-group-h">
        <div>
          <b>{{ t("groupWorkbench") }}</b>
          <span>{{ t("groupWorkbenchSub") }}</span>
        </div>
      </div>
      <div class="cfg-group-b">
        <div class="cfg-row">
          <div>
            <div class="cfg-label">{{ t("actionButtonCount") }}</div>
            <div class="cfg-hint">{{ t("actionButtonCountHint") }}</div>
          </div>
          <a-input-number
            :min="1"
            :max="10"
            :precision="0"
            :value="settings.general.actionButtonCount"
            @change="onActionButtonCountChange"
          />
        </div>
        <div class="cfg-row" style="margin-top: 14px">
          <div>
            <div class="cfg-label">{{ t("healthCheckOnScan") }}</div>
            <div class="cfg-hint">{{ t("healthCheckOnScanHint") }}</div>
          </div>
          <a-switch
            :checked="settings.general.healthCheckOnScan"
            @change="onHealthCheckOnScanChange"
          />
        </div>
      </div>
    </div>

    <div class="cfg-group">
      <div class="cfg-group-h">
        <div>
          <b>{{ t("groupExternalTools") }}</b>
          <span>{{ t("groupExternalToolsSub") }}</span>
        </div>
      </div>
      <div class="cfg-group-b">
        <div class="cfg-row">
          <div>
            <div class="cfg-label">{{ t("editorCommand") }}</div>
            <div class="cfg-hint">{{ t("editorCommandHint") }}</div>
          </div>
          <a-select
            :value="editorSelectValue"
            :options="editorOptions"
            style="min-width: 200px; max-width: 280px"
            :placeholder="t('editorCommandPlaceholder')"
            @change="onEditorCommandChange"
          />
        </div>
        <div class="cfg-row" style="margin-top: 14px">
          <div>
            <div class="cfg-label">{{ t("terminalApp") }}</div>
            <div class="cfg-hint">{{ t("terminalAppHint") }}</div>
          </div>
          <a-select
            :value="terminalSelectValue"
            :options="terminalOptions"
            style="min-width: 200px; max-width: 280px"
            :placeholder="t('terminalAppPlaceholder')"
            @change="onTerminalAppChange"
          />
        </div>
      </div>
    </div>

    <div class="cfg-group">
      <div class="cfg-group-h">
        <div>
          <b>{{ t("groupWorkspaceConfig") }}</b>
          <span>{{ t("groupWorkspaceConfigSub") }}</span>
        </div>
      </div>
      <div class="cfg-group-b">
        <p class="cfg-hint workspace-config-help">{{ t("workspaceConfigHelp") }}</p>
        <div class="cfg-row" style="margin-top: 10px">
          <div>
            <div class="cfg-label">{{ t("preferWorkspaceConfig") }}</div>
            <div class="cfg-hint">{{ t("preferWorkspaceConfigHint") }}</div>
          </div>
          <a-switch
            :checked="settings.general.preferWorkspaceConfig"
            @change="onPreferWorkspaceConfigChange"
          />
        </div>
        <div class="cfg-row" style="margin-top: 14px">
          <div>
            <div class="cfg-label">{{ t("workspaceConfigExport") }}</div>
            <div class="cfg-hint">{{ t("workspaceConfigExportHint") }}</div>
          </div>
          <a-button size="small" @click="onExportWorkspaceConfig">
            {{ t("workspaceConfigExportBtn") }}
          </a-button>
        </div>
      </div>
    </div>

    <div class="cfg-group">
      <div class="cfg-group-h">
        <div>
          <b>{{ t("groupApp") }}</b>
          <span>{{ t("groupAppSub") }}</span>
        </div>
      </div>
      <div class="cfg-group-b">
        <div class="cfg-row">
          <div>
            <div class="cfg-label">{{ t("launchAtLogin") }}</div>
            <div class="cfg-hint">{{ t("launchAtLoginHint") }}</div>
          </div>
          <a-switch
            :checked="settings.general.launchAtLogin"
            @change="onLaunchAtLoginChange"
          />
        </div>
        <div class="cfg-row" style="margin-top: 14px">
          <div>
            <div class="cfg-label">{{ t("closeToTray") }}</div>
            <div class="cfg-hint">{{ t("closeToTrayHint") }}</div>
          </div>
          <a-switch
            :checked="settings.general.closeToTray"
            @change="onCloseToTrayChange"
          />
        </div>
        <div class="cfg-row" style="margin-top: 14px">
          <div>
            <div class="cfg-label">{{ t("autoCheckUpdate") }}</div>
            <div class="cfg-hint">{{ t("autoCheckUpdateHint") }}</div>
          </div>
          <a-switch
            :checked="settings.general.autoCheckUpdate"
            @change="onAutoCheckUpdateChange"
          />
        </div>
        <div class="cfg-row" style="margin-top: 14px">
          <div>
            <div class="cfg-label">{{ t("notifyOnTaskDone") }}</div>
            <div class="cfg-hint">{{ t("notifyOnTaskDoneHint") }}</div>
          </div>
          <a-switch
            :checked="settings.general.notifyOnTaskDone"
            @change="onNotifyOnTaskDoneChange"
          />
        </div>
        <div class="cfg-row" style="margin-top: 14px">
          <div>
            <div class="cfg-label">{{ t("appVersion") }}</div>
            <div class="cfg-hint">{{ t("appVersionHint", { version: appVersionLabel }) }}</div>
          </div>
          <a-badge :dot="updateAvailable" :offset="[-2, 2]">
            <a-button
              size="small"
              :loading="updateChecking"
              :disabled="!appUpdate"
              @click="onCheckUpdateClick"
            >
              {{ t("checkForUpdate") }}
            </a-button>
          </a-badge>
        </div>
      </div>
    </div>
  </section>
</template>
