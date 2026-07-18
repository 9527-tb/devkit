<!--
  应用内更新确认：版本说明、进度、安装前实例提示。
-->
<script setup>
import { computed } from "vue";
import { createTranslator } from "../../i18n/index.js";
import { locale } from "../../stores/settings.js";

const props = defineProps({
  open: { type: Boolean, default: false },
  version: { type: String, default: "" },
  notes: { type: String, default: "" },
  runningCount: { type: Number, default: 0 },
  downloading: { type: Boolean, default: false },
  downloadPercent: { type: Number, default: 0 },
});

const emit = defineEmits(["update:open", "later", "install"]);

const t = createTranslator(locale);

const progressStatus = computed(() =>
  props.downloading && props.downloadPercent >= 100 ? "success" : "active",
);
</script>

<template>
  <a-modal
    :open="open"
    :title="t('updateAvailableTitle')"
    :closable="!downloading"
    :mask-closable="false"
    :keyboard="!downloading"
    :footer="null"
    @update:open="$emit('update:open', $event)"
    @cancel="$emit('later')"
  >
    <p class="update-version">
      {{ t("updateAvailableBody", { version }) }}
    </p>
    <pre v-if="notes" class="update-notes">{{ notes }}</pre>
    <p v-if="runningCount > 0 && !downloading" class="update-warn">
      {{ t("updateStopInstancesHint", { n: runningCount }) }}
    </p>
    <div v-if="downloading" class="update-progress">
      <a-progress
        :percent="downloadPercent"
        :status="progressStatus"
        size="small"
      />
      <p class="update-progress-hint">{{ t("updateDownloading") }}</p>
    </div>
    <div class="update-actions">
      <a-button :disabled="downloading" @click="$emit('later')">
        {{ t("updateLater") }}
      </a-button>
      <a-button
        type="primary"
        :loading="downloading"
        @click="$emit('install')"
      >
        {{ t("updateNow") }}
      </a-button>
    </div>
  </a-modal>
</template>

<style scoped>
.update-version {
  margin: 0 0 12px;
  color: var(--dk-text);
}

.update-notes {
  margin: 0 0 12px;
  max-height: 180px;
  overflow: auto;
  padding: 10px 12px;
  border-radius: var(--radius);
  background: var(--dk-panel-muted, rgba(0, 0, 0, 0.04));
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-word;
}

.update-warn {
  margin: 0 0 12px;
  color: var(--dk-warn, #b45309);
  font-size: 12px;
}

.update-progress {
  margin-bottom: 12px;
}

.update-progress-hint {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--dk-muted, #667);
}

.update-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}
</style>
