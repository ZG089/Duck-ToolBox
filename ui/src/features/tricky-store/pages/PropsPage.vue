<script setup lang="ts">
import { useMutation, useQueryClient } from "@tanstack/vue-query"
import { Save } from "@lucide/vue"
import { computed, ref, watch } from "vue"

import { Button } from "@/components/ui/button"
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldError,
  FieldLabel,
} from "@/components/ui/field"
import { Spinner } from "@/components/ui/spinner"
import { Switch } from "@/components/ui/switch"
import { Textarea } from "@/components/ui/textarea"
import QueryState from "@/core/components/QueryState.vue"
import SectionCard from "@/core/components/SectionCard.vue"
import { notifySuccess } from "@/core/notify"

import { trickyStoreApi } from "../api"
import { useTrickyI18n } from "../i18n"
import { STATUS_KEY, useStatusQuery } from "../queries"

const { t } = useTrickyI18n()
const client = useQueryClient()
const status = useStatusQuery()
const handler = ref(true)
const bootHash = ref("")

watch(
  status.data,
  (next) => {
    if (!next) return
    handler.value = next.props.prop_handler_enabled
    bootHash.value = next.props.boot_hash ?? ""
  },
  { immediate: true },
)

const hash = computed(() => bootHash.value.trim().toLowerCase())
const invalid = computed(() => hash.value !== "" && !/^[0-9a-f]{64}$/.test(hash.value))

const save = useMutation({
  mutationFn: () =>
    trickyStoreApi.saveProps({
      prop_handler_enabled: handler.value,
      boot_hash: hash.value || null,
    }),
  meta: { errorTitle: () => t("ta.prompt_boot_hash_set_error") },
  onSuccess: (result) => {
    notifySuccess(
      t("ta.prompt_boot_hash_set"),
      result.synced_backend_policy ? t("props.synced") : undefined,
    )
    void client.invalidateQueries({ queryKey: STATUS_KEY })
  },
})
</script>

<template>
  <div class="flex flex-col gap-4 p-4">
    <QueryState
      :loading="status.isPending.value"
      :error="status.error.value"
      @retry="status.refetch()"
    >
      <SectionCard>
        <div class="flex flex-col gap-6">
          <Field orientation="horizontal">
            <FieldContent>
              <FieldLabel for="prop-handler">{{ t("ta.prop_handler") }}</FieldLabel>
              <FieldDescription>{{ t("props.handlerHint") }}</FieldDescription>
            </FieldContent>
            <Switch id="prop-handler" v-model="handler" />
          </Field>
          <Field :data-invalid="invalid || undefined">
            <FieldLabel for="boot-hash">{{ t("ta.boot_hash_title") }}</FieldLabel>
            <Textarea
              id="boot-hash"
              v-model="bootHash"
              rows="3"
              class="font-mono text-xs"
              placeholder="241890bd44131d34c077cb01a0c3ea1ff68533b21e9d83b3f3adca6663c3d443"
              autocapitalize="off"
              spellcheck="false"
              :aria-invalid="invalid"
            />
            <FieldError v-if="invalid">{{ t("props.invalidHash") }}</FieldError>
            <FieldDescription v-else>{{ t("props.bootHashHint") }}</FieldDescription>
          </Field>
        </div>
      </SectionCard>
      <Button :disabled="invalid || save.isPending.value" @click="save.mutate()">
        <Spinner v-if="save.isPending.value" />
        <Save v-else />
        {{ t("ta.functional_button_save") }}
      </Button>
    </QueryState>
  </div>
</template>
