<!--
  设置「基本配置」面板：日志换行、主题、语言等。
  依赖：stores/settings、themes、i18n、useSettings。
  对应 DESIGN.md §12.2 SettingsGeneral
-->
<script setup>
import { inject, computed } from "vue";
import { THEME_OPTIONS, THEME_LABEL_KEYS } from "../../themes/index.js";
import { LOCALE_OPTIONS, createTranslator } from "../../i18n/index.js";
import { locale, settings } from "../../stores/settings.js";
import { useSettings } from "./useSettings.js";

// TODO(fe-settings-general): DONE — 从 App.vue 迁出基本配置表单 — DESIGN §12.2

const t = createTranslator(locale);
const {
  onThemeChange,
  onLocaleChange,
  onLogWrapSettingChange,
  onActionButtonCountChange,
  onLaunchAtLoginChange,
  onCloseToTrayChange,
  onAutoCheckUpdateChange,
} = useSettings();

const appUpdate = inject("appUpdate", null);
const updateChecking = computed(() => !!appUpdate?.checking?.value);
const updateAvailable = computed(() => !!appUpdate?.updateAvailable?.value);
const appVersionLabel = computed(() => appUpdate?.appVersion?.value || "—");

function onCheckUpdateClick() {
  appUpdate?.checkForUpdate?.({ silent: false });
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
