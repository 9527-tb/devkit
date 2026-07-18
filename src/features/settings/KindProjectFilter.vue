<!--
  单 Kind 项目过滤 checkbox 块，嵌入对应工具链设置页。
-->
<script setup>
import { computed } from "vue";
import { createTranslator } from "../../i18n/index.js";
import { locale, patchSettings, settings } from "../../stores/settings.js";
import {
  FILTER_ACTION_OPTIONS,
  setMavenOnlyJar,
  toggleKindFilterAction,
} from "./projectFilter.js";

const props = defineProps({
  /** maven | node | cargo */
  kind: { type: String, required: true },
  titleKey: { type: String, required: true },
  hintKey: { type: String, required: true },
});

const t = createTranslator(locale);

const options = computed(() => {
  const selected = new Set(settings.value.projectFilter?.[props.kind]?.actions || []);
  return (FILTER_ACTION_OPTIONS[props.kind] || []).map((opt) => ({
    ...opt,
    checked: selected.has(opt.value),
  }));
});

const selectedCount = computed(
  () => settings.value.projectFilter?.[props.kind]?.actions?.length || 0,
);

const onlyJar = computed(
  () => !!settings.value.projectFilter?.maven?.onlyJar,
);

function onToggle(action, checkedOrEvent) {
  const checked =
    typeof checkedOrEvent === "boolean"
      ? checkedOrEvent
      : !!checkedOrEvent?.target?.checked;
  patchSettings((s) => toggleKindFilterAction(s, props.kind, action, checked));
}

function onOnlyJar(checkedOrEvent) {
  const checked =
    typeof checkedOrEvent === "boolean"
      ? checkedOrEvent
      : !!checkedOrEvent?.target?.checked;
  patchSettings((s) => setMavenOnlyJar(s, checked));
}
</script>

<template>
  <div class="cfg-group">
    <div class="cfg-group-h">
      <div>
        <b>{{ t(titleKey) }}</b>
        <span>{{ t(hintKey) }}</span>
      </div>
    </div>
    <div class="cfg-group-b">
      <div class="filter-checks">
        <a-checkbox
          v-if="kind === 'maven'"
          :checked="onlyJar"
          @change="onOnlyJar"
        >
          {{ t("projectsFilterOnlyJar") }}
        </a-checkbox>
        <a-checkbox
          v-for="opt in options"
          :key="opt.value"
          :checked="opt.checked"
          @change="(e) => onToggle(opt.value, e)"
        >
          {{ t("projectsFilterRequire", { action: opt.label }) }}
        </a-checkbox>
      </div>
      <p class="cfg-hint">
        {{
          selectedCount === 0
            ? t("projectsFilterHintAll")
            : t("projectsFilterHintActive", { n: selectedCount })
        }}
      </p>
    </div>
  </div>
</template>

<style scoped>
.filter-checks {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 5px;
}
.filter-checks :deep(.ant-checkbox-wrapper) {
  font-size: 12px;
  font-weight: 600;
  color: var(--ink-strong);
}
</style>
