<!--
  SSH 部署 MVP 面板。
  对应 docs/DEV_PRODUCTIVITY_DESIGN.md ⑬
-->
<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { message } from "antdv-next";
import { createTranslator } from "../../i18n/index.js";
import { locale, previewMode } from "../../stores/settings.js";

defineProps({
  project: { type: Object, default: null },
});

const t = createTranslator(locale);

const localPath = ref("");
const host = ref("");
const user = ref("");
const remotePath = ref("");
const remoteCmd = ref("");
const identityFile = ref("");
const deploying = ref(false);
const deployLogs = ref([]);

async function runDeploy() {
  if (previewMode.value) {
    message.info(t("previewNeedTauri"));
    return;
  }
  if (!localPath.value.trim() || !host.value.trim() || !user.value.trim() || !remotePath.value.trim()) {
    message.warning(t("deployFieldsRequired"));
    return;
  }
  deploying.value = true;
  deployLogs.value = [];
  try {
    const result = await invoke("deploy_ssh_upload", {
      request: {
        localPath: localPath.value.trim(),
        host: host.value.trim(),
        user: user.value.trim(),
        remotePath: remotePath.value.trim(),
        remoteCmd: remoteCmd.value.trim() || null,
        identityFile: identityFile.value.trim() || null,
      },
    });
    deployLogs.value = result.logs || [];
    if (result.ok) {
      message.success(t("deploySuccess"));
    } else {
      message.error(result.error || t("deployFail"));
    }
  } catch (error) {
    message.error(String(error));
  } finally {
    deploying.value = false;
  }
}
</script>

<template>
  <section class="settings-panel deploy-panel">
    <h2>{{ t("deployTitle") }}</h2>
    <p class="settings-desc">{{ t("deployDescMvp") }}</p>

    <div class="cfg-group">
      <div class="cfg-group-b deploy-form">
        <div class="cfg-row">
          <div>
            <div class="cfg-label">{{ t("deployLocalPath") }}</div>
          </div>
          <a-input v-model:value="localPath" :placeholder="t('deployLocalPathHint')" />
        </div>
        <div class="cfg-row" style="margin-top: 10px">
          <div>
            <div class="cfg-label">{{ t("deployHost") }}</div>
          </div>
          <a-input v-model:value="host" placeholder="192.168.1.10" />
        </div>
        <div class="cfg-row" style="margin-top: 10px">
          <div>
            <div class="cfg-label">{{ t("deployUser") }}</div>
          </div>
          <a-input v-model:value="user" placeholder="deploy" />
        </div>
        <div class="cfg-row" style="margin-top: 10px">
          <div>
            <div class="cfg-label">{{ t("deployRemotePath") }}</div>
          </div>
          <a-input v-model:value="remotePath" placeholder="/var/www/app" />
        </div>
        <div class="cfg-row" style="margin-top: 10px">
          <div>
            <div class="cfg-label">{{ t("deployIdentityFile") }}</div>
            <div class="cfg-hint">{{ t("deployIdentityFileHint") }}</div>
          </div>
          <a-input v-model:value="identityFile" placeholder="~/.ssh/id_rsa" />
        </div>
        <div class="cfg-row" style="margin-top: 10px">
          <div>
            <div class="cfg-label">{{ t("deployRemoteCmd") }}</div>
            <div class="cfg-hint">{{ t("deployRemoteCmdHint") }}</div>
          </div>
          <a-input v-model:value="remoteCmd" placeholder="systemctl restart myapp" />
        </div>
        <div class="deploy-actions">
          <a-button type="primary" :loading="deploying" @click="runDeploy">
            {{ t("deployRun") }}
          </a-button>
        </div>
      </div>
    </div>

    <div v-if="deployLogs.length" class="deploy-log">
      <h4>{{ t("deployLog") }}</h4>
      <pre>{{ deployLogs.join("\n") }}</pre>
    </div>
  </section>
</template>

<style scoped>
.deploy-panel h2 {
  margin: 0 0 8px;
}
.deploy-form .cfg-row {
  display: grid;
  grid-template-columns: 160px 1fr;
  gap: 12px;
  align-items: start;
}
.deploy-actions {
  margin-top: 16px;
}
.deploy-log {
  margin-top: 16px;
  border: 1px solid var(--line, #d7e0dc);
  border-radius: var(--radius);
  overflow: hidden;
}
.deploy-log h4 {
  margin: 0;
  padding: 8px 12px;
  font-size: 12px;
  background: var(--input-bg, #f4f8f6);
  border-bottom: 1px solid var(--line, #d7e0dc);
}
.deploy-log pre {
  margin: 0;
  padding: 10px 12px;
  max-height: 240px;
  overflow: auto;
  font-size: 11px;
  line-height: 1.45;
  font-family: "IBM Plex Mono", ui-monospace, monospace;
}
</style>
