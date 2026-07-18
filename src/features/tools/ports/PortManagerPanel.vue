<!--
  端口管理面板（tools/ports 专用）。
-->
<script setup>
import { computed, ref } from "vue";
import { message, Modal } from "antdv-next";
import { SearchOutlined } from "@antdv-next/icons";
import { lookupPort, killPid } from "./api.js";
import { createTranslator } from "../../../i18n/index.js";
import { locale } from "../../../stores/settings.js";

const t = createTranslator(locale);
const portInput = ref(null);
const fuzzy = ref(false);
const loading = ref(false);
const killing = ref(null);
const rows = ref([]);
const searched = ref(false);
const error = ref("");

const columns = computed(() => [
  { title: t("portManagerColPort"), dataIndex: "port", key: "port", width: 90 },
  { title: t("portManagerColPid"), dataIndex: "pid", key: "pid", width: 100 },
  { title: t("portManagerColName"), dataIndex: "name", key: "name", ellipsis: true },
  { title: t("portManagerColUser"), dataIndex: "user", key: "user", width: 120, ellipsis: true },
  { title: t("portManagerColState"), dataIndex: "state", key: "state", width: 120 },
  { title: t("portManagerColAddress"), dataIndex: "address", key: "address", width: 200, ellipsis: true },
  { title: t("portManagerColAction"), key: "action", width: 100, fixed: "right" },
]);

function resolvedQuery() {
  const raw = String(portInput.value ?? "").trim();
  if (!raw || !/^\d{1,5}$/.test(raw)) return null;
  if (!fuzzy.value) {
    const n = Number(raw);
    if (!Number.isInteger(n) || n < 1 || n > 65535) return null;
  }
  return raw;
}

function rowKey(record) {
  return `${record.pid}-${record.port}`;
}

async function onQuery() {
  const query = resolvedQuery();
  if (query == null) {
    message.warning(
      fuzzy.value ? t("portManagerInvalidQuery") : t("portManagerInvalidPort"),
    );
    return;
  }
  loading.value = true;
  error.value = "";
  searched.value = true;
  try {
    const list = await lookupPort(query, fuzzy.value);
    rows.value = Array.isArray(list) ? list : [];
    if (!rows.value.length) {
      message.info(t("portManagerEmpty"));
    }
  } catch (e) {
    rows.value = [];
    error.value = String(e);
    message.error(error.value);
  } finally {
    loading.value = false;
  }
}

function onKill(row) {
  Modal.confirm({
    title: t("portManagerKillConfirmTitle"),
    content: t("portManagerKillConfirmBody", {
      pid: row.pid,
      name: row.name || "—",
    }),
    okText: t("portManagerKill"),
    okType: "danger",
    cancelText: t("cancel"),
    async onOk() {
      killing.value = row.pid;
      try {
        await killPid(row.pid);
        message.success(t("portManagerKilled", { pid: row.pid }));
        await onQuery();
      } catch (e) {
        message.error(String(e));
        throw e;
      } finally {
        killing.value = null;
      }
    },
  });
}
</script>

<template>
  <section class="settings-panel">
    <h2>{{ t("portManagerTitle") }}</h2>
    <p class="settings-desc">{{ t("portManagerHint") }}</p>

    <div class="cfg-group">
      <div class="cfg-group-h">
        <div>
          <b>{{ t("portManagerQueryGroup") }}</b>
          <span>{{ t("portManagerQueryGroupSub") }}</span>
        </div>
      </div>
      <div class="cfg-group-b">
        <div class="port-query-row">
          <a-input-number
            v-model:value="portInput"
            class="port-input"
            :min="1"
            :max="65535"
            :placeholder="t('portManagerPlaceholder')"
            :controls="false"
            @press-enter="onQuery"
          />
          <a-checkbox v-model:checked="fuzzy">{{ t("portManagerFuzzy") }}</a-checkbox>
          <a-button type="primary" :loading="loading" @click="onQuery">
            <template #icon><SearchOutlined /></template>
            {{ t("portManagerQuery") }}
          </a-button>
        </div>
        <a-alert v-if="error" type="error" show-icon :message="error" class="port-err" />
      </div>
    </div>

    <div class="cfg-group">
      <div class="cfg-group-h">
        <div>
          <b>{{ t("portManagerResultGroup") }}</b>
          <span>{{ t("portManagerResultGroupSub") }}</span>
        </div>
      </div>
      <div class="cfg-group-b port-result-b">
        <a-table
          size="small"
          :loading="loading"
          :columns="columns"
          :data-source="rows"
          :pagination="false"
          :row-key="rowKey"
          :locale="{ emptyText: searched ? t('portManagerEmpty') : t('portManagerIdle') }"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.key === 'action'">
              <a-button
                type="primary"
                danger
                :loading="killing === record.pid"
                @click="onKill(record)"
              >
                {{ t("portManagerKill") }}
              </a-button>
            </template>
          </template>
        </a-table>
      </div>
    </div>
  </section>
</template>

<style scoped>
.port-query-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  max-width: 560px;
}
.port-input {
  width: 160px;
}
.port-err {
  margin-top: 10px;
}
.port-result-b {
  padding: 0;
  overflow: hidden;
}
.port-result-b :deep(.ant-table) {
  background: transparent;
}
.port-result-b :deep(.ant-table-wrapper) {
  margin: 0;
}
</style>
