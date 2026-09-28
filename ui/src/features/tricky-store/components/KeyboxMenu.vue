<script setup lang="ts">
import { computed, ref } from "vue"
import { FileKey2, FolderOpen, LoaderCircle, RefreshCcw, ShieldCheck } from "@lucide/vue"

import { useI18n } from "@/i18n"
import { filesCommand } from "@/lib/api/tricky-store"
import type { FileEntry } from "../types"
import type { TrickyStoreController } from "../useTrickyStore"

const props = defineProps<{ controller: TrickyStoreController }>()
const { t } = useI18n()
const { state, actions } = props.controller

const fetchUrl = ref("")
const fetchDecode = ref("")
const pickerOpen = ref(false)
const pickerPath = ref("/storage/emulated/0/Download")
const pickerEntries = ref<FileEntry[]>([])
const pickerParent = ref<string | null>(null)
const pickerBusy = ref(false)

const keybox = computed(() => state.status?.keybox ?? null)

function humanTime(unix?: number) {
  return unix ? new Date(unix * 1000).toLocaleString() : t("trickyStore.notAvailable")
}

async function openPicker() {
  pickerOpen.value = true
  await browse(pickerPath.value)
}

async function browse(path: string) {
  pickerBusy.value = true
  try {
    const envelope = await filesCommand(path, "xml")
    if (envelope.ok && envelope.data) {
      pickerPath.value = envelope.data.path
      pickerEntries.value = envelope.data.entries
      pickerParent.value = envelope.data.parent ?? null
    }
  } finally {
    pickerBusy.value = false
  }
}

async function choose(entry: FileEntry) {
  if (entry.directory) {
    await browse(entry.path)
    return
  }
  pickerOpen.value = false
  await actions.installKeybox("local", { path: entry.path })
}

async function fetchFromUrl() {
  if (!fetchUrl.value.trim()) return
  await actions.installKeybox("fetch", { url: fetchUrl.value, decode: fetchDecode.value })
}

function useProvider(url: string, decode: string) {
  fetchUrl.value = url
  fetchDecode.value = decode
  fetchFromUrl()
}
</script>

<template>
  <section class="panel-subsection">
    <div class="panel-heading compact">
      <div>
        <div class="section-kicker">{{ t("trickyStore.keyboxKicker") }}</div>
        <h3 class="subheading">{{ t("trickyStore.keyboxTitle") }}</h3>
      </div>
      <FileKey2 class="icon-muted" />
    </div>

    <article class="summary-tile keybox-meta">
      <span class="summary-label">{{ t("trickyStore.currentKeybox") }}</span>
      <p class="mono-inline mt-2 break-all">{{ keybox?.path ?? t("trickyStore.notAvailable") }}</p>
      <p class="muted mt-2">
        <template v-if="keybox?.exists">{{ keybox.size }} bytes · {{ humanTime(keybox?.modified_unix) }}</template>
        <template v-else>{{ t("trickyStore.keyboxMissing") }}</template>
      </p>
    </article>

    <div class="toolbar mt-4">
      <button class="action-secondary" type="button" :disabled="state.busy.keybox" @click="actions.installKeybox('aosp')">
        <ShieldCheck class="size-4" />
        {{ t("trickyStore.keyboxAosp") }}
      </button>
      <button class="action-secondary" type="button" :disabled="state.busy.keybox" @click="actions.installKeybox('generate')">
        <FileKey2 class="size-4" />
        {{ t("trickyStore.keyboxUnknown") }}
      </button>
      <button class="action-secondary" type="button" :disabled="state.busy.keybox" @click="openPicker">
        <FolderOpen class="size-4" />
        {{ t("trickyStore.keyboxLocal") }}
      </button>
      <LoaderCircle v-if="state.busy.keybox" class="size-4 animate-spin self-center" />
    </div>

    <div class="field-grid two-up mt-4">
      <div class="field-group">
        <label class="field-label" for="keybox-url">{{ t("trickyStore.keyboxUrl") }}</label>
        <div class="copy-row">
          <input id="keybox-url" v-model="fetchUrl" class="text-input" placeholder="https://…/keybox.xml">
          <button class="icon-button" type="button" :disabled="state.busy.keybox" :title="t('trickyStore.keyboxFetch')" @click="fetchFromUrl">
            <RefreshCcw class="size-4" />
          </button>
        </div>
      </div>
      <div class="field-group">
        <label class="field-label" for="keybox-decode">{{ t("trickyStore.keyboxDecode") }}</label>
        <input id="keybox-decode" v-model="fetchDecode" class="text-input" placeholder="xxd -r -p | base64 -d">
      </div>
    </div>

    <div v-if="state.providers.length" class="chip-row mt-4">
      <button
        v-for="provider in state.providers"
        :key="provider.name"
        class="option-chip"
        type="button"
        :disabled="state.busy.keybox"
        @click="useProvider(provider.url, provider.decode)"
      >
        {{ provider.name }}
      </button>
      <button class="option-chip" type="button" :disabled="state.busy.providers" @click="actions.resetProviders()">
        {{ t("functional.reset") }}
      </button>
    </div>

    <Teleport to="body">
      <div v-if="pickerOpen" class="dialog-backdrop" @click.self="pickerOpen = false">
        <section class="dialog-card file-browser-dialog" role="dialog" aria-modal="true">
          <div class="panel-heading compact">
            <div>
              <h3 class="panel-title dialog-title">{{ t("trickyStore.chooseKeybox") }}</h3>
              <p class="mono-inline mt-2 break-all">{{ pickerPath }}</p>
            </div>
          </div>
          <div class="toolbar">
            <button class="action-secondary" type="button" :disabled="!pickerParent || pickerBusy" @click="pickerParent && browse(pickerParent)">..</button>
            <button class="action-secondary" type="button" :disabled="pickerBusy" @click="browse(pickerPath)">
              <LoaderCircle v-if="pickerBusy" class="size-4 animate-spin" />
              <RefreshCcw v-else class="size-4" />
              {{ t("actions.refreshTrickyStore") }}
            </button>
          </div>
          <div class="stack-list file-browser-list">
            <button
              v-for="entry in pickerEntries"
              :key="entry.path"
              class="list-row file-browser-row"
              type="button"
              @click="choose(entry)"
            >
              <span class="copy-row">
                <FolderOpen v-if="entry.directory" class="size-4 icon-muted" />
                <FileKey2 v-else class="size-4 icon-muted" />
                <span>
                  <span class="subheading">{{ entry.name }}</span>
                  <span class="mono-inline muted target-package">{{ entry.path }}</span>
                </span>
              </span>
            </button>
          </div>
          <p v-if="!pickerEntries.length && !pickerBusy" class="empty-copy mt-4">{{ t("trickyStore.noKeyboxFiles") }}</p>
          <div class="toolbar mt-4">
            <button class="action-secondary" type="button" @click="pickerOpen = false">{{ t("dialog.close") }}</button>
          </div>
        </section>
      </div>
    </Teleport>
  </section>
</template>
