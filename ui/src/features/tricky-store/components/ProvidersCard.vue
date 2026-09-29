<script setup lang="ts">
import { useMutation, useQueryClient } from "@tanstack/vue-query"
import { CloudDownload, Download, Pencil, Plus, RotateCcw, Upload } from "@lucide/vue"
import { computed, ref } from "vue"

import { Button } from "@/components/ui/button"
import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemGroup,
  ItemTitle,
} from "@/components/ui/item"
import { Spinner } from "@/components/ui/spinner"
import ConfirmDialog from "@/core/components/ConfirmDialog.vue"
import FilePickerDialog from "@/core/components/FilePickerDialog.vue"
import type { PickedFile } from "@/core/components/FilePickerDialog.vue"
import QueryState from "@/core/components/QueryState.vue"
import SectionCard from "@/core/components/SectionCard.vue"
import { notifySuccess } from "@/core/notify"

import type { Provider } from "../api"
import { trickyStoreApi } from "../api"
import { useTrickyI18n } from "../i18n"
import { useKeyboxInstall } from "../keybox"
import { useProvidersQuery } from "../queries"
import ProviderDialog from "./ProviderDialog.vue"

const { t } = useTrickyI18n()
const client = useQueryClient()
const providers = useProvidersQuery()
const list = computed(() => providers.data.value ?? [])

const editing = ref<number | null>(null)
const editorOpen = ref(false)
const confirm = ref<"remove" | "reset" | null>(null)
const importing = ref(false)

const setList = (next: Provider[]) => client.setQueryData(["tricky-store", "providers"], next)

const fetchKeybox = useKeyboxInstall(
  (provider: Provider) => trickyStoreApi.keyboxFetch(provider.url, provider.decode),
  { success: "ta.prompt_custom_key_set", failure: "ta.prompt_custom_fetch_error" },
)

const saveList = useMutation({ mutationFn: trickyStoreApi.saveProviders, onSuccess: setList })

function edit(index: number | null) {
  editing.value = index
  editorOpen.value = true
}

function save(provider: Provider) {
  const next = [...list.value]
  if (editing.value === null) next.push(provider)
  else next[editing.value] = provider
  saveList.mutate(next, {
    onSuccess: () => {
      editorOpen.value = false
      notifySuccess(t("ta.prompt_custom_saved"))
    },
  })
}

function remove() {
  const index = editing.value
  if (index === null) return
  saveList.mutate(
    list.value.filter((_, position) => position !== index),
    {
      onSuccess: () => {
        editorOpen.value = false
        notifySuccess(t("ta.prompt_custom_removed"))
      },
    },
  )
}

const reset = useMutation({
  mutationFn: trickyStoreApi.resetProviders,
  onSuccess: (next) => {
    setList(next)
    notifySuccess(t("ta.prompt_custom_removed"))
  },
})

const importFile = useMutation({
  mutationFn: (file: PickedFile) =>
    "path" in file
      ? trickyStoreApi.importProviders(file.path)
      : trickyStoreApi.importProvidersContent(file.content),
  meta: { errorTitle: () => t("ta.customkb_import_error") },
  onSuccess: (next) => {
    setList(next)
    notifySuccess(t("ta.customkb_import_success"))
  },
})

const exportFile = useMutation({
  mutationFn: trickyStoreApi.exportProviders,
  meta: { errorTitle: () => t("ta.customkb_export_error") },
  onSuccess: (result) => notifySuccess(t("ta.customkb_export_success", [result.path])),
})

function exportProviders() {
  if (!list.value.length) notifySuccess(t("ta.customkb_export_empty"))
  else exportFile.mutate()
}

function confirmed() {
  if (confirm.value === "remove") remove()
  else if (confirm.value === "reset") reset.mutate()
}
</script>

<template>
  <SectionCard :title="t('keybox.custom')" plain>
    <template #action>
      <Button size="sm" variant="secondary" @click="edit(null)">
        <Plus />
        {{ t("keybox.addProvider") }}
      </Button>
    </template>

    <QueryState
      :loading="providers.isPending.value"
      :error="providers.error.value"
      @retry="providers.refetch()"
    >
      <p v-if="!list.length" class="bg-card text-muted-foreground rounded-lg p-4 text-sm">
        {{ t("keybox.customEmpty") }}
      </p>
      <ItemGroup v-else>
        <Item
          v-for="(provider, index) in list"
          :key="`${provider.name}-${index}`"
          variant="segmented"
          class="pe-3"
        >
          <ItemContent class="min-w-0">
            <ItemTitle>{{ provider.name }}</ItemTitle>
            <ItemDescription dir="ltr" class="truncate font-mono text-xs rtl:text-right">{{
              provider.url
            }}</ItemDescription>
          </ItemContent>
          <ItemActions class="gap-2">
            <Button
              variant="ghost"
              size="icon"
              :aria-label="t('keybox.editProvider')"
              @click="edit(index)"
            >
              <Pencil class="size-5" />
            </Button>
            <Button
              size="icon"
              :aria-label="provider.name"
              :disabled="fetchKeybox.isPending.value"
              @click="fetchKeybox.mutate(provider)"
            >
              <Spinner
                v-if="fetchKeybox.isPending.value && fetchKeybox.variables.value === provider"
              />
              <CloudDownload v-else class="size-5" />
            </Button>
          </ItemActions>
        </Item>
      </ItemGroup>
    </QueryState>

    <div class="mt-3 flex flex-wrap gap-2">
      <Button variant="outline" @click="importing = true">
        <Upload />
        {{ t("keybox.import") }}
      </Button>
      <Button variant="outline" :disabled="exportFile.isPending.value" @click="exportProviders">
        <Download />
        {{ t("keybox.export") }}
      </Button>
      <Button variant="ghost" class="ms-auto" @click="confirm = 'reset'">
        <RotateCcw />
        {{ t("keybox.resetProviders") }}
      </Button>
    </div>

    <ProviderDialog
      v-model:open="editorOpen"
      :provider="editing === null ? null : (list[editing] ?? null)"
      :saving="saveList.isPending.value"
      @save="save"
      @remove="confirm = 'remove'"
    />
    <ConfirmDialog
      :open="confirm !== null"
      :title="t('ta.customkb_remove_title')"
      :description="
        confirm === 'reset' ? t('ta.customkb_reset_message') : t('ta.customkb_remove_message')
      "
      :confirm-label="
        confirm === 'reset' ? t('ta.functional_button_reset') : t('ta.functional_button_remove')
      "
      destructive
      @update:open="!$event && (confirm = null)"
      @confirm="confirmed"
    />
    <FilePickerDialog v-model:open="importing" extension="json" @pick="importFile.mutate($event)" />
  </SectionCard>
</template>
