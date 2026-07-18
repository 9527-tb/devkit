<!--
  工具链设置面板：Java/JDK/Maven 与 Node（过渡；R5 改为注册表驱动）。
  依赖：stores/settings、useSettings。
  对应 DESIGN.md §12.2 SettingsToolchain
-->
<script setup>
import { computed, onMounted, ref, watch } from "vue";
import { Empty } from "antdv-next";
import { DeleteOutlined, PlusOutlined } from "@antdv-next/icons";
import { createTranslator } from "../../i18n/index.js";
import { detectNodePackageManagers } from "../../api/settings.js";
import {
  locale,
  settings,
  settingsCat,
  entryKey,
  NODE_PACKAGE_MANAGERS,
  normalizeNodePackageManager,
  previewMode,
} from "../../stores/settings.js";
import { useSettings } from "./useSettings.js";
import KindProjectFilter from "./KindProjectFilter.vue";

// DONE(fe-settings-java): Java/JDK/Maven 设置迁出 App.vue — DESIGN §12.2
// DONE(fe-settings-node): Node 设置迁出 App.vue — DESIGN §12.2
// DONE(fe-settings-toolchain-dynamic): 导航按 list_providers；本面板按 cat 渲染 — DESIGN R5

const emptySimpleImage = Empty.PRESENTED_IMAGE_SIMPLE;
const t = createTranslator(locale);
const {
  addRuntimePath,
  pickMavenHome,
  removeRuntime,
  onMavenHomeInput,
  onNodePackageManagerChange,
} = useSettings();

/** 系统已检测到的包管理器 id 列表 */
const installedPackageManagers = ref([...NODE_PACKAGE_MANAGERS]);

const packageManagerSelectOptions = computed(() => {
  const ids = installedPackageManagers.value.filter((x) =>
    NODE_PACKAGE_MANAGERS.includes(x),
  );
  const current = normalizeNodePackageManager(settings.value.node?.packageManager);
  const list = ids.length && !ids.includes(current) ? [current, ...ids] : ids;
  return list.map((pm) => ({ value: pm, label: pm }));
});

async function refreshPackageManagers() {
  if (previewMode.value) {
    installedPackageManagers.value = [...NODE_PACKAGE_MANAGERS];
    return;
  }
  try {
    const found = await detectNodePackageManagers();
    const list = (Array.isArray(found) ? found : [])
      .map((x) => String(x).toLowerCase())
      .filter((x) => NODE_PACKAGE_MANAGERS.includes(x));
    // 探测失败或为空时回退完整列表，避免下拉无选项
    installedPackageManagers.value = list.length ? list : [...NODE_PACKAGE_MANAGERS];
    const current = normalizeNodePackageManager(settings.value.node?.packageManager);
    if (list.length && !list.includes(current)) {
      onNodePackageManagerChange(list[0]);
    }
  } catch (error) {
    console.warn("[settings] detect_node_package_managers failed", error);
    installedPackageManagers.value = [...NODE_PACKAGE_MANAGERS];
  }
}

onMounted(() => {
  refreshPackageManagers();
});

watch(settingsCat, (cat) => {
  if (cat === "node") refreshPackageManagers();
});
</script>

<template>
  <section v-show="settingsCat === 'java'" class="settings-panel">
    <h2>{{ t("javaTitle") }}</h2>
    <p class="settings-desc">{{ t("javaDesc") }}</p>

    <div class="cfg-group">
      <div class="cfg-group-h">
        <div>
          <b>JDK</b>
          <span>{{ t("jdkVersions", { n: settings.java.jdks.length }) }}</span>
        </div>
        <a-button @click="addRuntimePath('jdk')">
          <template #icon><PlusOutlined /></template>
        </a-button>
      </div>
      <div class="cfg-group-b">
        <a-empty
          v-if="!settings.java.jdks.length"
          :image="emptySimpleImage"
          :description="t('noJdk')"
        />
        <ul v-else class="runtime-list">
          <li v-for="item in settings.java.jdks" :key="entryKey(item)">
            <div>
              <strong>{{ item.label || `JDK ${item.version}` }}</strong>
              <small>{{ item.path }}</small>
            </div>
            <a-button type="text" danger @click="removeRuntime('jdk', entryKey(item))">
              <template #icon><DeleteOutlined /></template>
              {{ t("delete") }}
            </a-button>
          </li>
        </ul>
      </div>
    </div>

    <div class="cfg-group">
      <div class="cfg-group-h">
        <div>
          <b>Maven</b>
          <span>{{ t("mavenUseHint") }}</span>
        </div>
      </div>
      <div class="cfg-group-b">
        <div class="cfg-label">{{ t("mavenHome") }}</div>
        <div class="cfg-hint">{{ t("mavenHomeHint") }}</div>
        <div class="runtime-add">
          <a-input
            :value="settings.java.mavenHome"
            placeholder="/opt/homebrew/opt/maven/libexec"
            @update:value="onMavenHomeInput"
          />
          <a-button @click="pickMavenHome">{{ t("pick") }}</a-button>
        </div>
      </div>
    </div>

    <KindProjectFilter
      kind="maven"
      title-key="mavenProjectFilter"
      hint-key="projectsFilterMavenHint"
    />
  </section>

  <section v-show="settingsCat === 'node'" class="settings-panel">
    <h2>{{ t("nodeTitle") }}</h2>
    <p class="settings-desc">{{ t("nodeDesc") }}</p>
    <div class="cfg-group">
      <div class="cfg-group-h">
        <div>
          <b>{{ t("versionManage") }}</b>
          <span>{{ t("nodeVersions", { n: settings.node.nodes.length }) }}</span>
        </div>
        <a-button @click="addRuntimePath('node')">
          <template #icon><PlusOutlined /></template>
        </a-button>
      </div>
      <div class="cfg-group-b">
        <a-empty
          v-if="!settings.node.nodes.length"
          :image="emptySimpleImage"
          :description="t('noNode')"
        />
        <ul v-else class="runtime-list">
          <li v-for="item in settings.node.nodes" :key="entryKey(item)">
            <div>
              <strong>{{ item.label || `Node ${item.version}` }}</strong>
              <small>{{ item.path }}</small>
            </div>
            <a-button type="text" danger @click="removeRuntime('node', entryKey(item))">
              <template #icon><DeleteOutlined /></template>
              {{ t("delete") }}
            </a-button>
          </li>
        </ul>
      </div>
    </div>

    <div class="cfg-group">
      <div class="cfg-group-h">
        <div>
          <b>{{ t("nodePackageManager") }}</b>
          <span>{{ t("nodePackageManagerHint") }}</span>
        </div>
      </div>
      <div class="cfg-group-b">
        <a-select
          v-if="packageManagerSelectOptions.length"
          :value="settings.node.packageManager"
          style="width: 200px"
          :options="packageManagerSelectOptions"
          @update:value="onNodePackageManagerChange"
        />
        <p v-else class="cfg-hint">{{ t("nodePackageManagerEmpty") }}</p>
      </div>
    </div>

    <KindProjectFilter
      kind="node"
      title-key="nodeProjectFilter"
      hint-key="projectsFilterNodeHint"
    />
  </section>

  <section v-show="settingsCat === 'cargo'" class="settings-panel">
    <h2>{{ t("navCargo") }}</h2>
    <p class="settings-desc">
      {{ t("cargoDesc") }}
    </p>

    <KindProjectFilter
      kind="cargo"
      title-key="cargoProjectFilter"
      hint-key="projectsFilterCargoHint"
    />
  </section>

  <section v-show="settingsCat === 'gradle'" class="settings-panel">
    <h2>{{ t("navGradle") }}</h2>
    <p class="settings-desc">
      {{ t("gradleDesc") }}
    </p>

    <KindProjectFilter
      kind="gradle"
      title-key="gradleProjectFilter"
      hint-key="projectsFilterGradleHint"
    />
  </section>
</template>
