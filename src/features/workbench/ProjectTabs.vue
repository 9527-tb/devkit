<!--
  多项目标签栏：位于主内容区顶部（侧栏右侧），对齐原型样式。
  对应 DESIGN.md §10.4 / prototype ProjectTabs
-->
<script setup>
import { nextTick, onMounted, onBeforeUnmount, ref, watch } from "vue";
import { LeftOutlined, RightOutlined, UnorderedListOutlined } from "@antdv-next/icons";
import { createTranslator } from "../../i18n/index.js";
import { locale } from "../../stores/settings.js";

// DONE(ux-project-tabs): 多项目标签 + 左右滚动 + 跳转 — DESIGN §10.4

const props = defineProps({
  tabs: { type: Array, default: () => [] },
  activeKey: { type: String, default: "" },
  /** (tab) => boolean 是否运行中 */
  isRunning: { type: Function, default: () => false },
});

const emit = defineEmits(["activate", "close", "jump", "context-action"]);

const t = createTranslator(locale);

function tabContextMenu(tab) {
  const idx = props.tabs.findIndex((t) => t.key === tab.key);
  const othersCount = idx >= 0 ? props.tabs.length - 1 : 0;
  const rightCount = idx >= 0 ? props.tabs.length - idx - 1 : 0;
  const idleCount = props.tabs.filter((t) => !props.isRunning(t)).length;
  const runningCount = props.tabs.filter((t) => props.isRunning(t)).length;
  return {
    items: [
      { key: "all", label: t("closeTabAll"), disabled: !props.tabs.length },
      { key: "others", label: t("closeTabOthers"), disabled: othersCount === 0 },
      { key: "right", label: t("closeTabRight"), disabled: rightCount === 0 },
      { type: "divider" },
      { key: "idle", label: t("closeTabIdle"), disabled: idleCount === 0 },
      { key: "running", label: t("closeTabRunning"), disabled: runningCount === 0 },
    ],
    onClick: ({ key }) => {
      if (!key || key === "divider") return;
      emit("context-action", { action: String(key), tab });
    },
  };
}

const barEl = ref(null);
const scrollEl = ref(null);
const overflow = ref(false);
const canPrev = ref(false);
const canNext = ref(false);
const jumpOpen = ref(false);

function kindLabel(kind) {
  if (kind === "maven") return "Maven";
  if (kind === "node") return "Node";
  if (kind === "cargo") return "Cargo";
  if (kind === "gradle") return "Gradle";
  return kind || "";
}

function updateNavState() {
  const el = scrollEl.value;
  if (!el) {
    canPrev.value = false;
    canNext.value = false;
    return;
  }
  const max = Math.max(0, el.scrollWidth - el.clientWidth);
  overflow.value = props.tabs.length > 0 && el.scrollWidth > el.clientWidth + 2;
  canPrev.value = el.scrollLeft > 2;
  canNext.value = el.scrollLeft < max - 2;
}

function scrollBy(dir) {
  const el = scrollEl.value;
  if (!el) return;
  el.scrollBy({ left: dir * Math.max(160, el.clientWidth * 0.6), behavior: "smooth" });
}

function scrollActiveIntoView() {
  const el = scrollEl.value;
  if (!el) return;
  const tab = el.querySelector(".project-tab.active");
  if (!tab) return;
  const left = tab.offsetLeft;
  const right = left + tab.offsetWidth;
  const viewLeft = el.scrollLeft;
  const viewRight = viewLeft + el.clientWidth;
  if (left < viewLeft + 8) {
    el.scrollTo({ left: Math.max(0, left - 24), behavior: "smooth" });
  } else if (right > viewRight - 8) {
    el.scrollTo({ left: right - el.clientWidth + 24, behavior: "smooth" });
  }
}

watch(
  () => [props.tabs.length, props.activeKey],
  async () => {
    await nextTick();
    updateNavState();
    scrollActiveIntoView();
  },
);

onMounted(() => {
  updateNavState();
  window.addEventListener("resize", updateNavState);
  document.addEventListener("click", onDocClick);
});
onBeforeUnmount(() => {
  window.removeEventListener("resize", updateNavState);
  document.removeEventListener("click", onDocClick);
});

function onDocClick(e) {
  if (!barEl.value?.contains(e.target)) jumpOpen.value = false;
}

function onJump(key) {
  jumpOpen.value = false;
  emit("jump", key);
}
</script>

<template>
  <div
    ref="barEl"
    class="project-tabs-bar"
    :class="{ 'is-overflow': overflow }"
  >
    <button
      type="button"
      class="tabs-nav"
      :title="t('tabsScrollLeft')"
      :disabled="!canPrev"
      @click="scrollBy(-1)"
    >
      <LeftOutlined />
    </button>

    <div class="project-tabs-scroll">
      <div
        ref="scrollEl"
        class="project-tabs"
        @scroll="updateNavState"
      >
        <div v-if="!tabs.length" class="project-tabs-empty">{{ t("tabsEmpty") }}</div>
        <a-dropdown
          v-for="tab in tabs"
          :key="tab.key"
          :trigger="['contextmenu']"
          :menu="tabContextMenu(tab)"
        >
          <button
            type="button"
            class="project-tab"
            :class="{ active: tab.key === activeKey }"
            @click="emit('activate', tab.key)"
          >
            <span class="dot" :class="{ on: isRunning(tab) }" />
            <span class="label" :title="tab.name">{{ tab.name }}</span>
            <span
              class="close"
              :title="t('close')"
              :aria-label="t('close')"
              @click.stop="emit('close', tab)"
            >×</span>
          </button>
        </a-dropdown>
      </div>
    </div>

    <button
      type="button"
      class="tabs-nav"
      :title="t('tabsScrollRight')"
      :disabled="!canNext"
      @click="scrollBy(1)"
    >
      <RightOutlined />
    </button>

    <div class="tabs-jump" :class="{ open: jumpOpen }">
      <button
        type="button"
        class="tabs-jump-btn"
        :title="t('tabsJumpAll')"
        @click.stop="jumpOpen = !jumpOpen"
      >
        <UnorderedListOutlined />
      </button>
      <div v-show="jumpOpen && tabs.length" class="tabs-jump-menu">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          type="button"
          class="tabs-jump-item"
          :class="{ active: tab.key === activeKey }"
          @click="onJump(tab.key)"
        >
          <span class="dot" :class="{ on: isRunning(tab) }" />
          <span class="name">{{ tab.name }}</span>
          <span class="kind">{{ kindLabel(tab.kind) }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.project-tabs-bar {
  flex: none;
  display: flex;
  align-items: center;
  gap: 2px;
  height: 38px;
  min-height: 38px;
  max-height: 38px;
  padding: 0 6px;
  box-sizing: border-box;
  background: color-mix(in srgb, var(--panel, #fff) 92%, transparent);
  border-bottom: 1px solid var(--line, #d7e0dc);
}
.tabs-nav,
.tabs-jump-btn {
  flex: none;
  width: 26px;
  height: 26px;
  border-radius: var(--radius);
  display: none;
  place-items: center;
  color: var(--muted, #6b7a76);
  background: transparent;
  border: 0;
  cursor: pointer;
  font-size: 11px;
  transition: background 0.12s, color 0.12s, opacity 0.12s;
}
.tabs-nav:hover:not(:disabled),
.tabs-jump-btn:hover,
.tabs-jump.open .tabs-jump-btn {
  background: var(--nav-hover, #e8f1ed);
  color: var(--teal, #0f766e);
}
.tabs-nav:disabled {
  opacity: 0.28;
  cursor: default;
}
.project-tabs-bar.is-overflow .tabs-nav,
.project-tabs-bar.is-overflow .tabs-jump-btn {
  display: grid;
}
.project-tabs-scroll {
  flex: 1;
  min-width: 0;
  overflow: hidden;
}
.project-tabs {
  display: flex;
  align-items: center;
  gap: 2px;
  height: 38px;
  min-height: 38px;
  max-height: 38px;
  overflow-x: auto;
  overflow-y: hidden;
  scroll-behavior: smooth;
}
.project-tabs :deep(.ant-dropdown-trigger) {
  flex: none;
  display: inline-flex;
  align-items: center;
  height: 26px;
  max-height: 26px;
  line-height: 0;
}
.project-tabs::-webkit-scrollbar {
  height: 0;
}
.project-tab {
  flex: none;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  max-width: 160px;
  width: max-content;
  height: 26px;
  min-height: 26px;
  max-height: 26px;
  padding: 0 6px 0 9px;
  border: 0;
  border-radius: var(--radius);
  box-sizing: border-box;
  overflow: hidden;
  color: var(--muted, #6b7a76);
  font-size: 12px;
  font-weight: 600;
  font-family: inherit;
  line-height: 26px;
  background: var(--surface-tint, #e8efec);
  cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.project-tab:hover {
  background: var(--chip-bg, #dce8e3);
  color: var(--ink, #15201e);
}
.project-tab.active {
  color: var(--teal, #0f766e);
  background: var(--teal-soft, #e8f5f2);
}
/* 纯色圆点，固定盒模型；启停只改颜色，不改尺寸/光晕 */
.dot {
  width: 6px;
  height: 6px;
  min-width: 6px;
  min-height: 6px;
  max-width: 6px;
  max-height: 6px;
  flex: none;
  border-radius: 50%;
  box-sizing: border-box;
  background: var(--line-hover, #b9cdc5);
  box-shadow: none;
  transition: background 0.12s ease;
}
.dot.on {
  background: var(--running, #149a6a);
}
.label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 26px;
}
.close {
  flex: none;
  width: 16px;
  height: 16px;
  min-width: 16px;
  min-height: 16px;
  margin-left: 1px;
  border-radius: var(--radius);
  display: grid;
  place-items: center;
  color: var(--muted, #6b7a76);
  font-size: 14px;
  line-height: 1;
  opacity: 0;
}
.project-tab:hover .close,
.project-tab.active .close {
  opacity: 0.85;
}
.close:hover {
  opacity: 1;
  background: color-mix(in srgb, var(--teal, #0f766e) 14%, transparent);
  color: var(--teal, #0f766e);
}
.project-tabs-empty {
  display: flex;
  align-items: center;
  height: 38px;
  padding: 0 4px;
  color: var(--muted, #6b7a76);
  font-size: 12px;
  font-weight: 500;
}
.tabs-jump {
  position: relative;
  flex: none;
}
.tabs-jump-menu {
  position: absolute;
  right: 0;
  top: calc(100% + 6px);
  z-index: 30;
  width: min(240px, 70vw);
  max-height: 260px;
  overflow: auto;
  padding: 4px;
  border-radius: var(--radius);
  border: 1px solid var(--line, #d7e0dc);
  background: #fff;
  box-shadow: 0 10px 24px rgba(19, 32, 28, 0.1);
}
.tabs-jump-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border: 0;
  border-radius: var(--radius);
  text-align: left;
  color: var(--ink-soft, #30433e);
  font-size: 12px;
  font-weight: 600;
  font-family: inherit;
  background: transparent;
  cursor: pointer;
}
.tabs-jump-item:hover {
  background: var(--surface-hover, #f3f7f5);
}
.tabs-jump-item.active {
  background: var(--teal-soft, #e8f5f2);
  color: var(--teal, #0f766e);
}
.tabs-jump-item .name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.tabs-jump-item .kind {
  flex: none;
  color: var(--muted, #6b7a76);
  font-size: 10px;
  font-weight: 600;
}
.tabs-jump-item.active .kind {
  color: color-mix(in srgb, var(--teal, #0f766e) 65%, #8a9a94);
}
</style>
