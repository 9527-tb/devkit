<!--
  工具链 + 项目类型设置面板（按 settingsCat 渲染）。
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
  SETTINGS_CAT,
} from "../../stores/settings.js";
import { useSettings } from "./useSettings.js";
import KindProjectFilter from "./KindProjectFilter.vue";

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
  if (cat === SETTINGS_CAT.TOOLCHAIN_NODE) refreshPackageManagers();
});
</script>

<template>
  <!-- 工具链：JDK -->
  <section v-show="settingsCat === SETTINGS_CAT.TOOLCHAIN_JDK" class="settings-panel">
    <h2>{{ t("jdkTitle") }}</h2>
    <p class="settings-desc">{{ t("jdkDesc") }}</p>

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
  </section>

  <!-- 工具链：Node.js -->
  <section v-show="settingsCat === SETTINGS_CAT.TOOLCHAIN_NODE" class="settings-panel">
    <h2>{{ t("toolchainNodeTitle") }}</h2>
    <p class="settings-desc">{{ t("toolchainNodeDesc") }}</p>
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
  </section>

  <!-- 项目类型：Maven -->
  <section v-show="settingsCat === SETTINGS_CAT.PROVIDER_MAVEN" class="settings-panel">
    <h2>{{ t("mavenTitle") }}</h2>
    <p class="settings-desc">{{ t("mavenDesc") }}</p>

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

  <!-- 项目类型：Gradle -->
  <section v-show="settingsCat === SETTINGS_CAT.PROVIDER_GRADLE" class="settings-panel">
    <h2>{{ t("gradleTitle") }}</h2>
    <p class="settings-desc">{{ t("gradleDesc") }}</p>

    <KindProjectFilter
      kind="gradle"
      title-key="gradleProjectFilter"
      hint-key="projectsFilterGradleHint"
    />
  </section>

  <!-- 项目类型：Node 项目 -->
  <section v-show="settingsCat === SETTINGS_CAT.PROVIDER_NODE" class="settings-panel">
    <h2>{{ t("providerNodeTitle") }}</h2>
    <p class="settings-desc">{{ t("providerNodeDesc") }}</p>

    <KindProjectFilter
      kind="node"
      title-key="nodeProjectFilter"
      hint-key="projectsFilterNodeHint"
    />
  </section>

  <!-- 项目类型：Cargo -->
  <section v-show="settingsCat === SETTINGS_CAT.PROVIDER_CARGO" class="settings-panel">
    <h2>{{ t("cargoTitle") }}</h2>
    <p class="settings-desc">{{ t("cargoDesc") }}</p>

    <KindProjectFilter
      kind="cargo"
      title-key="cargoProjectFilter"
      hint-key="projectsFilterCargoHint"
    />
  </section>
</template>
