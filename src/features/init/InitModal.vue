<!--
  首次初始化进度模态：不可手动关闭，完成后自动关闭。
  依赖：stores/init、stores/settings；被 App 壳挂载。
  对应 DESIGN.md §6.1 / §12.2 InitModal
-->
<script setup>
import { computed } from "vue";
import { createTranslator } from "../../i18n/index.js";
import { locale, colorThemeId } from "../../stores/settings.js";
import { getTheme } from "../../themes/index.js";
import {
  INIT_STEP_DEFS,
  visible,
  percent,
  detail,
  done,
  stepStatus,
  onOpenChange,
} from "../../stores/init.js";

// TODO(fe-init-modal): DONE — 进度条、步骤列表与动态提示 UI — DESIGN §6.1

defineProps({
  primaryColor: { type: String, default: "" },
});

const t = createTranslator(locale);

const strokeColor = computed(() => {
  const pack = getTheme(colorThemeId.value);
  return pack.primary;
});
</script>

<template>
  <a-modal
    :open="visible"
    :title="t('initTitle')"
    :closable="false"
    :mask-closable="false"
    :keyboard="false"
    :footer="null"
    :centered="true"
    :destroy-on-hidden="true"
    width="440"
    wrap-class-name="init-modal-wrap"
    :z-index="2000"
    @update:open="onOpenChange"
  >
    <p class="init-hint">{{ t("initHint") }}</p>
    <a-progress
      :percent="percent"
      :status="done ? 'success' : 'active'"
      :stroke-color="primaryColor || strokeColor"
    />
    <ul class="init-steps">
      <li
        v-for="item in INIT_STEP_DEFS"
        :key="item.id"
        :class="stepStatus[item.id]"
      >
        <span class="init-step-mark" aria-hidden="true" />
        <span class="init-step-label">{{ t(item.labelKey) }}</span>
      </li>
    </ul>
    <p class="init-detail">{{ detail || t("initWait") }}</p>
    <p class="init-wait">{{ t("initWait") }}</p>
  </a-modal>
</template>

<style>
.init-hint {
  margin: 0 0 14px;
  color: var(--muted);
  font-size: 12px;
  line-height: 1.5;
}
.init-steps {
  list-style: none;
  margin: 14px 0 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.init-steps li {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  font-weight: 600;
  color: var(--muted);
}
.init-steps li.active { color: var(--teal); }
.init-steps li.done { color: var(--ink-strong); }
.init-step-mark {
  width: 8px;
  height: 8px;
  border-radius: 100%;
  background: var(--line);
  flex: none;
}
.init-steps li.active .init-step-mark {
  background: var(--teal);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--teal) 22%, transparent);
}
.init-steps li.done .init-step-mark {
  background: var(--running);
}
.init-detail {
  margin: 12px 0 0;
  min-height: 18px;
  color: var(--ink-soft);
  font-size: 12px;
  font-weight: 500;
}
.init-wait {
  margin: 8px 0 0;
  color: var(--muted);
  font-size: 11px;
}
.init-modal-wrap .ant-modal-content {
  border-radius: var(--radius);
  overflow: hidden;
}
.init-modal-wrap .ant-modal-header {
  border-bottom: 1px solid var(--line-soft);
}
.init-modal-wrap .ant-modal-mask {
  backdrop-filter: blur(2px);
}
</style>
