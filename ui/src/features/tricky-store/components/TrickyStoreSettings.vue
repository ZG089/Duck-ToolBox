<script setup lang="ts">
import { useMutation, useQueryClient } from "@tanstack/vue-query"
import { Languages, Send } from "@lucide/vue"
import { computed, ref } from "vue"

import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Field, FieldContent, FieldDescription, FieldLabel } from "@/components/ui/field"
import { Separator } from "@/components/ui/separator"
import { Spinner } from "@/components/ui/spinner"
import { Switch } from "@/components/ui/switch"
import MarkdownView from "@/core/components/MarkdownView.vue"
import SectionCard from "@/core/components/SectionCard.vue"
import { openExternal, REPOSITORY_URL } from "@/core/links"
import { notifySuccess } from "@/core/notify"

import { trickyStoreApi } from "../api"
import { useTrickyI18n } from "../i18n"
import { useEntryQuery } from "../queries"
import { translationsVersion, updateTranslations } from "../translation-update"

/** Tricky Addon's channel, from its About dialog. */
const TELEGRAM_CHANNEL = "https://t.me/kowchannel"

const { t } = useTrickyI18n()
const client = useQueryClient()
const entry = useEntryQuery()
const guideOpen = ref(false)
const version = ref(translationsVersion())

const toggle = useMutation({
  mutationFn: trickyStoreApi.setEntry,
  onSuccess: (next) => client.setQueryData(["tricky-store", "entry"], next),
})

// As upstream: announce the result, then reload so every page picks the strings up.
const update = useMutation({
  mutationFn: updateTranslations,
  meta: { errorTitle: () => t("ta.prompt_translation_update_failed") },
  onSuccess: (result) => {
    if (result === "latest") {
      notifySuccess(t("ta.prompt_no_update"))
      return
    }
    version.value = translationsVersion()
    notifySuccess(t("ta.prompt_translation_updated"))
    setTimeout(() => location.reload(), 1200)
  },
})

const module = computed(() => entry.data.value?.module_id ?? "")
const description = computed(() => {
  const data = entry.data.value
  if (!data?.module_id) return t("entry.noModule")
  if (data.has_own_webui) return t("entry.ownWebui", { module: data.module_id })
  return t("entry.description", { module: data.module_id })
})
</script>

<template>
  <SectionCard :title="t('meta.title')">
    <div class="flex flex-col gap-5">
      <Field v-if="entry.data.value" orientation="horizontal">
        <FieldContent>
          <FieldLabel for="tricky-store-entry">{{ t("entry.title") }}</FieldLabel>
          <FieldDescription>{{ description }}</FieldDescription>
        </FieldContent>
        <Switch
          id="tricky-store-entry"
          :model-value="entry.data.value.enabled"
          :disabled="!module || entry.data.value.has_own_webui || toggle.isPending.value"
          @update:model-value="toggle.mutate($event)"
        />
      </Field>
      <Separator v-if="entry.data.value" />

      <div class="flex items-center gap-3">
        <div class="min-w-0 flex-1">
          <div class="text-sm font-medium">{{ t("translations.title") }}</div>
          <div class="text-muted-foreground text-xs">
            {{ t("translations.version", { version }) }}
          </div>
        </div>
        <Button
          size="sm"
          variant="secondary"
          :disabled="update.isPending.value"
          @click="update.mutate()"
        >
          <Spinner v-if="update.isPending.value" />
          {{ t("ta.about_translation_update") }}
        </Button>
      </div>
      <div class="flex flex-wrap gap-2">
        <Button variant="outline" @click="guideOpen = true">
          <Languages />
          {{ t("translations.help") }}
        </Button>
        <Button variant="outline" @click="openExternal(TELEGRAM_CHANNEL)">
          <Send />
          {{ t("translations.telegram") }}
        </Button>
      </div>
    </div>

    <Dialog v-model:open="guideOpen">
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{{ t("translations.help") }}</DialogTitle>
        </DialogHeader>
        <MarkdownView :source="t('translations.guide', { repository: REPOSITORY_URL })" />
      </DialogContent>
    </Dialog>
  </SectionCard>
</template>
