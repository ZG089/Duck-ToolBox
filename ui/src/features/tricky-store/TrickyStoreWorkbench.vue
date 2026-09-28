<script setup lang="ts">
import { computed, ref } from "vue"
import {
  LoaderCircle,
  Plus,
  RefreshCcw,
  Save,
  Settings2,
  ShieldCheck,
  Trash2,
} from "@lucide/vue"

import { useI18n } from "@/i18n"
import TextPreviewDialog from "@/features/shared/TextPreviewDialog.vue"
import PolicyEditor from "./components/PolicyEditor.vue"
import TargetList from "./components/TargetList.vue"
import KeyboxMenu from "./components/KeyboxMenu.vue"
import PropsDialog from "./components/PropsDialog.vue"
import { useTrickyStore } from "./useTrickyStore"

const { t } = useI18n()
const controller = useTrickyStore()
const { state, actions, backendName, backendInstalled } = controller

const systemAppInput = ref("")
const propsOpen = ref(false)

const backendStatusLabel = computed(() =>
  backendInstalled.value ? t("trickyStore.installed") : t("trickyStore.notInstalled"),
)
const unavailable = computed(() => state.bridge.mode === "unavailable")

function addSystemApp() {
  actions.addSystemApp(systemAppInput.value)
  systemAppInput.value = ""
}
</script>

<template>
  <section class="panel">
    <div class="panel-heading">
      <div>
        <div class="section-kicker">{{ t("tool.workspaceKicker") }}</div>
        <h2 class="panel-title">{{ t("tool.trickyStoreName") }}</h2>
        <p class="body-copy mt-3 max-w-3xl">{{ t("tool.trickyStoreDescription") }}</p>
      </div>
    </div>

    <p v-if="unavailable" class="error-banner">{{ t("messages.ksuUnavailable") }}</p>

    <div class="toolbar">
      <button
        class="action-primary"
        :disabled="state.busy.save || !backendInstalled || unavailable"
        @click="actions.save()"
      >
        <LoaderCircle v-if="state.busy.save" class="size-4 animate-spin" />
        <Save v-else class="size-4" />
        {{ t("actions.saveTrickyStoreTargets") }}
      </button>
      <button class="action-secondary" :disabled="state.busy.status || unavailable" @click="actions.refresh()">
        <LoaderCircle v-if="state.busy.status" class="size-4 animate-spin" />
        <RefreshCcw v-else class="size-4" />
        {{ t("actions.refreshTrickyStore") }}
      </button>
      <button class="action-secondary" :disabled="!backendInstalled || unavailable" @click="propsOpen = true">
        <Settings2 class="size-4" />
        {{ t("trickyStore.propsTitle") }}
      </button>
    </div>

    <div class="summary-grid wide">
      <article class="summary-tile">
        <div class="panel-heading compact">
          <span class="summary-label">{{ backendName }}</span>
          <ShieldCheck :class="['icon-muted', backendInstalled ? 'status-icon-online' : 'status-icon-offline']" />
        </div>
        <p class="mono-inline mt-2">{{ backendStatusLabel }}</p>
        <p class="muted mt-2 break-all">{{ state.status?.active?.module_dir ?? t("trickyStore.noBackend") }}</p>
        <p v-if="state.status?.active?.version" class="muted mt-1">{{ state.status.active.version }}</p>
      </article>
      <article class="summary-tile">
        <span class="summary-label">{{ t("trickyStore.detectedBackends") }}</span>
        <div class="chip-row mt-2">
          <span v-for="backend in state.status?.backends ?? []" :key="backend.module_id" class="tool-pill">
            {{ backend.identity }}<template v-if="!backend.active"> · {{ t("trickyStore.inactive") }}</template>
          </span>
          <span v-if="!state.status?.backends.length" class="muted">{{ t("trickyStore.noBackend") }}</span>
        </div>
      </article>
    </div>

    <section v-if="backendInstalled && state.schema" class="panel-subsection">
      <div class="panel-heading compact">
        <div>
          <div class="section-kicker">{{ t("trickyStore.defaultPolicyKicker") }}</div>
          <h3 class="subheading">{{ t("trickyStore.defaultPolicyTitle") }}</h3>
        </div>
      </div>
      <PolicyEditor
        :fields="state.schema.default_policy"
        :model-value="state.defaultPolicy"
        @update:model-value="state.defaultPolicy = $event"
      />
    </section>

    <TargetList v-if="backendInstalled" :controller="controller" />

    <section v-if="backendInstalled" class="panel-subsection">
      <div class="panel-heading compact">
        <div>
          <div class="section-kicker">{{ t("trickyStore.quickSelectKicker") }}</div>
          <h3 class="subheading">{{ t("trickyStore.quickSelectTitle") }}</h3>
        </div>
      </div>
      <div class="toolbar">
        <button class="action-secondary" :disabled="state.busy.apps" @click="actions.loadApps('unnecessary')">
          {{ t("trickyStore.deselectUnnecessary") }}
        </button>
        <button class="action-secondary" :disabled="state.busy.apps" @click="actions.loadApps('unnecessary', true)">
          <RefreshCcw class="size-4" />
          {{ t("trickyStore.refreshUnnecessary") }}
        </button>
        <button class="action-secondary" :disabled="state.busy.apps" @click="actions.loadApps('xposed')">
          {{ t("trickyStore.selectXposed") }}
        </button>
        <button
          v-if="state.status?.root_manager?.kind === 'magisk'"
          class="action-secondary"
          :disabled="state.busy.apps"
          @click="actions.loadApps('denylist')"
        >
          {{ t("trickyStore.selectDenylist") }}
        </button>
      </div>
    </section>

    <KeyboxMenu v-if="backendInstalled" :controller="controller" />

    <section v-if="backendInstalled" class="panel-subsection">
      <div class="panel-heading compact">
        <div>
          <div class="section-kicker">{{ t("trickyStore.systemAppsKicker") }}</div>
          <h3 class="subheading">{{ t("trickyStore.systemAppsTitle") }}</h3>
        </div>
        <Plus class="icon-muted" />
      </div>
      <div class="copy-row">
        <input
          v-model="systemAppInput"
          class="text-input"
          placeholder="com.android.vending"
          @keyup.enter="addSystemApp()"
        >
        <button class="action-secondary" type="button" @click="addSystemApp()">
          <Plus class="size-4" />
          {{ t("actions.addSystemApp") }}
        </button>
      </div>
      <div v-if="state.systemApps.length" class="chip-row mt-4">
        <span v-for="packageName in state.systemApps" :key="packageName" class="system-chip">
          <span class="mono-inline">{{ packageName }}</span>
          <button class="icon-button compact-icon" type="button" @click="actions.removeSystemApp(packageName)">
            <Trash2 class="size-3" />
          </button>
        </span>
      </div>
      <p v-else class="empty-copy mt-4">{{ t("trickyStore.noSystemApps") }}</p>
    </section>

    <p v-else-if="!unavailable" class="empty-copy mt-4">{{ t("trickyStore.installBackendHint") }}</p>

    <PropsDialog :controller="controller" :open="propsOpen" @close="propsOpen = false" />

    <TextPreviewDialog
      :content="state.errorDialogText"
      :copy-label="t('actions.copyError')"
      :close-label="t('dialog.close')"
      :description="t('dialog.errorDescription')"
      :open="state.errorDialogOpen"
      :title="t('dialog.errorTitle')"
      @close="actions.dismissErrorDialog()"
      @copy="actions.copyText(state.errorDialogText)"
    />
  </section>
</template>
