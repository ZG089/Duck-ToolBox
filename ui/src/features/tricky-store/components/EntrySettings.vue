<script setup lang="ts">
import { useMutation, useQueryClient } from "@tanstack/vue-query"
import { computed } from "vue"

import { Field, FieldContent, FieldDescription, FieldLabel } from "@/components/ui/field"
import { Switch } from "@/components/ui/switch"
import SectionCard from "@/core/components/SectionCard.vue"

import { trickyStoreApi } from "../api"
import { useTrickyI18n } from "../i18n"
import { useEntryQuery } from "../queries"

/** Settings section: open this manager from the keystore module's own WebUI entry. */
const { t } = useTrickyI18n()
const client = useQueryClient()
const entry = useEntryQuery()

const toggle = useMutation({
  mutationFn: trickyStoreApi.setEntry,
  onSuccess: (next) => client.setQueryData(["tricky-store", "entry"], next),
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
  <SectionCard v-if="entry.data.value" :title="t('meta.title')">
    <Field orientation="horizontal">
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
  </SectionCard>
</template>
