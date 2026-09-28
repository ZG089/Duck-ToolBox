<script setup lang="ts">
import { computed, ref } from "vue"
import { Check, Search, SlidersHorizontal, Trash2 } from "@lucide/vue"

import { useI18n } from "@/i18n"
import PolicyEditor from "./PolicyEditor.vue"
import type { PackageScope, TrickyStoreController } from "../useTrickyStore"
import type { Policy, TargetMode } from "../types"

const props = defineProps<{ controller: TrickyStoreController }>()
const { t } = useI18n()

const { state, actions, visiblePackages, selectedCount } = props.controller

const modeEditorOpen = ref(false)
const modeTarget = ref<{ packageName: string; label: string } | null>(null)
const modeValue = ref<TargetMode>("auto")
const modePolicy = ref<Policy>({})

const scopeOptions = computed<Array<{ value: PackageScope; label: string }>>(() => [
  { value: "all", label: t("trickyStore.scopeAll") },
  { value: "user", label: t("trickyStore.scopeUser") },
  { value: "system", label: t("trickyStore.scopeSystem") },
])

const modeOptions = computed<Array<{ value: TargetMode; label: string }>>(() => [
  { value: "auto", label: t("trickyStore.modeAuto") },
  { value: "generate", label: t("trickyStore.modeGenerate") },
  { value: "hack", label: t("trickyStore.modeHack") },
])

const supportsMode = computed(() => state.schema?.supports_app_mode ?? false)
const supportsPerApp = computed(() => state.schema?.supports_per_app_policy ?? false)

function openModeEditor(packageName: string, label: string) {
  if (!supportsMode.value) return
  modeTarget.value = { packageName, label }
  modeValue.value = state.targets.get(packageName) ?? "auto"
  modePolicy.value = { ...(state.perAppPolicy[packageName] ?? {}) }
  modeEditorOpen.value = true
}

function applyMode() {
  if (!modeTarget.value) return
  const { packageName } = modeTarget.value
  actions.setMode(packageName, modeValue.value)
  if (supportsPerApp.value) {
    actions.setPerAppPolicy(packageName, modeValue.value === "auto" ? null : modePolicy.value)
  }
  modeEditorOpen.value = false
}

function modeBadge(mode: TargetMode): string {
  return mode === "generate" ? "!" : mode === "hack" ? "?" : ""
}
</script>

<template>
  <section class="panel-subsection">
    <div class="panel-heading compact">
      <div>
        <div class="section-kicker">{{ t("trickyStore.targetsKicker") }}</div>
        <h3 class="subheading">{{ t("trickyStore.targetsTitle") }}</h3>
      </div>
      <span class="mono-chip">{{ selectedCount }}</span>
    </div>

    <p v-if="supportsMode" class="security-note">
      <SlidersHorizontal class="size-4" />
      {{ t("trickyStore.modeNote") }}
    </p>

    <div class="toolbar tricky-toolbar">
      <label class="search-field">
        <Search class="size-4 icon-muted" />
        <input
          :value="state.search"
          class="search-input-native"
          :placeholder="t('trickyStore.searchPlaceholder')"
          @input="actions.setSearch(($event.target as HTMLInputElement).value)"
        >
      </label>
      <div class="segmented-control package-scope-control">
        <button
          v-for="option in scopeOptions"
          :key="option.value"
          :class="['segment-button', { 'is-active': state.scope === option.value }]"
          type="button"
          @click="actions.setScope(option.value)"
        >
          {{ option.label }}
        </button>
      </div>
      <button class="action-secondary" type="button" @click="actions.selectAllVisible(true)">
        <Check class="size-4" />
        {{ t("actions.selectAllVisible") }}
      </button>
      <button class="action-secondary" type="button" @click="actions.selectAllVisible(false)">
        <Trash2 class="size-4" />
        {{ t("actions.clearVisible") }}
      </button>
    </div>

    <label class="switch-row">
      <input
        type="checkbox"
        :checked="state.autoAddNewApps"
        @change="actions.setAutoAddNewApps(($event.target as HTMLInputElement).checked)"
      >
      <span>
        <strong>{{ t("trickyStore.autoAddNewApps") }}</strong>
        <small>{{ t("trickyStore.autoAddNewAppsDescription") }}</small>
      </span>
    </label>

    <div class="stack-list mt-4 app-target-list">
      <article
        v-for="entry in visiblePackages"
        :key="entry.package_name"
        class="list-row app-target-row"
        :class="{ 'is-selected': entry.selected }"
      >
        <button class="target-main" type="button" @click="actions.togglePackage(entry.package_name)">
          <span class="target-check" :class="{ 'is-on': entry.selected }">
            <Check v-if="entry.selected" class="size-4" />
          </span>
          <img
            class="app-icon"
            loading="lazy"
            alt=""
            :src="entry.iconUrl"
            @error="($event.target as HTMLImageElement).style.visibility = 'hidden'"
          >
          <span class="target-labels">
            <span class="subheading">{{ entry.label }}</span>
            <span class="mono-inline muted target-package">{{ entry.package_name }}</span>
            <span class="chip-row mt-2">
              <span class="tool-pill">{{ entry.system ? t("trickyStore.system") : t("trickyStore.user") }}</span>
              <span v-if="entry.selected && modeBadge(entry.mode)" class="tool-pill mode-pill">{{ modeBadge(entry.mode) }}</span>
            </span>
          </span>
        </button>
        <button
          v-if="supportsMode && entry.selected"
          class="icon-button"
          type="button"
          :title="t('trickyStore.editMode')"
          @click="openModeEditor(entry.package_name, entry.label)"
        >
          <SlidersHorizontal class="size-4" />
        </button>
      </article>
    </div>

    <p v-if="!visiblePackages.length" class="empty-copy mt-4">{{ t("trickyStore.noPackages") }}</p>

    <Teleport to="body">
      <div v-if="modeEditorOpen" class="dialog-backdrop" @click.self="modeEditorOpen = false">
        <section class="dialog-card" role="dialog" aria-modal="true">
          <div class="panel-heading compact">
            <div>
              <h3 class="panel-title dialog-title">{{ modeTarget?.label }}</h3>
              <p class="mono-inline mt-2 break-all">{{ modeTarget?.packageName }}</p>
            </div>
          </div>
          <div class="segmented-control mt-2">
            <button
              v-for="option in modeOptions"
              :key="option.value"
              :class="['segment-button', { 'is-active': modeValue === option.value }]"
              type="button"
              @click="modeValue = option.value"
            >
              {{ option.label }}
            </button>
          </div>
          <div v-if="supportsPerApp && modeValue !== 'auto' && state.schema" class="mt-4">
            <div class="section-kicker">{{ t("trickyStore.perAppPolicy") }}</div>
            <PolicyEditor
              class="mt-2"
              :fields="state.schema.default_policy"
              :model-value="modePolicy"
              @update:model-value="modePolicy = $event"
            />
          </div>
          <div class="toolbar mt-4">
            <button class="action-primary" type="button" @click="applyMode()">{{ t("functional.save") }}</button>
            <button class="action-secondary" type="button" @click="modeEditorOpen = false">{{ t("dialog.close") }}</button>
          </div>
        </section>
      </div>
    </Teleport>
  </section>
</template>
