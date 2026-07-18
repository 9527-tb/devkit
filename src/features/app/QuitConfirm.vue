<!--
  应用退出确认：存在运行中实例时三按钮选择。
-->
<script setup>
import { createTranslator } from "../../i18n/index.js";
import { locale } from "../../stores/settings.js";

defineProps({
  open: { type: Boolean, default: false },
  runningCount: { type: Number, default: 0 },
  loading: { type: Boolean, default: false },
});

const emit = defineEmits(["update:open", "cancel", "stop-and-quit", "quit-without-stop"]);

const t = createTranslator(locale);
</script>

<template>
  <a-modal
    :open="open"
    :title="t('quitConfirmTitle')"
    :closable="!loading"
    :mask-closable="false"
    :keyboard="!loading"
    :footer="null"
    @update:open="$emit('update:open', $event)"
    @cancel="$emit('cancel')"
  >
    <p>{{ t("quitConfirmBody", { n: runningCount }) }}</p>
    <div class="quit-actions">
      <a-button :disabled="loading" @click="$emit('cancel')">
        {{ t("quitCancel") }}
      </a-button>
      <a-button
        danger
        :loading="loading"
        @click="$emit('stop-and-quit')"
      >
        {{ t("quitStopAndExit") }}
      </a-button>
      <a-button
        type="primary"
        :disabled="loading"
        @click="$emit('quit-without-stop')"
      >
        {{ t("quitLeaveAndExit") }}
      </a-button>
    </div>
  </a-modal>
</template>

<style scoped>
.quit-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 20px;
}
</style>
