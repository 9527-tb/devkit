<!--
  关闭运行中项目 Tab 时的确认对话框（单标签 / 批量）。
  对应 DESIGN.md §10.4 / 原型
-->
<script setup>
import { createTranslator } from "../../i18n/index.js";
import { locale } from "../../stores/settings.js";

// DONE(ux-close-confirm): 关闭运行中确认 — DESIGN §10.4

defineProps({
  open: { type: Boolean, default: false },
  /** single | batch */
  mode: { type: String, default: "single" },
  projectName: { type: String, default: "" },
  /** 批量：将关闭的标签总数 */
  totalCount: { type: Number, default: 0 },
  /** 批量：其中仍在运行的数量 */
  runningCount: { type: Number, default: 0 },
});

defineEmits(["update:open", "confirm", "cancel"]);

const t = createTranslator(locale);
</script>

<template>
  <a-modal
    :open="open"
    :title="mode === 'batch' ? t('closeBatchTitle') : t('closeRunningTitle')"
    :ok-text="t('closeStopAndClose')"
    :cancel-text="t('cancel')"
    ok-type="danger"
    @update:open="$emit('update:open', $event)"
    @ok="$emit('confirm')"
    @cancel="$emit('cancel')"
  >
    <p v-if="mode === 'batch'">
      {{ t("closeBatchBody", { total: totalCount, running: runningCount }) }}
    </p>
    <p v-else>
      {{ t("closeRunningBody", { name: projectName }) }}
    </p>
  </a-modal>
</template>
