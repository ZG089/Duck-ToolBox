<script setup lang="ts">
import { useMutation, useQuery } from "@tanstack/vue-query"
import { ChevronDown, RefreshCw, Send } from "@lucide/vue"
import { computed, ref, watch } from "vue"

import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import { Field, FieldContent, FieldDescription, FieldLabel } from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { Spinner } from "@/components/ui/spinner"
import { Switch } from "@/components/ui/switch"
import ConfirmDialog from "@/core/components/ConfirmDialog.vue"
import KeyValueList from "@/core/components/KeyValueList.vue"
import QueryState from "@/core/components/QueryState.vue"
import SectionCard from "@/core/components/SectionCard.vue"
import { notifySuccess } from "@/core/notify"

import type { DeviceIdsProfile, DeviceIdsResult } from "../api"
import { deviceIdsApi } from "../api"
import { useDeviceIdsI18n } from "../i18n"

const { t } = useDeviceIdsI18n()
const defaults = useQuery({ queryKey: ["device-ids.defaults"], queryFn: deviceIdsApi.defaults })
const profile = ref<DeviceIdsProfile | null>(null)
const result = ref<DeviceIdsResult | null>(null)
const advancedOpen = ref(false)
const confirmOpen = ref(false)

watch(defaults.data, (data) => {
  if (data) profile.value = { ...data, dry_run: profile.value?.dry_run ?? true }
})

type Key = keyof Omit<DeviceIdsProfile, "dry_run">
const mainFields: Key[] = ["brand", "model", "device", "product", "manufacturer", "serial"]
const advancedFields: Key[] = ["imei", "imei2", "meid", "meid2", "ta_name", "ta_path"]
const labels: Record<Key, () => string> = {
  brand: () => t("form.brand"),
  model: () => t("form.model"),
  device: () => t("form.device"),
  product: () => t("form.product"),
  manufacturer: () => t("form.manufacturer"),
  serial: () => t("form.serial"),
  imei: () => t("form.imei"),
  imei2: () => t("form.imei2"),
  meid: () => t("form.meid"),
  meid2: () => t("form.meid2"),
  ta_name: () => t("form.taName"),
  ta_path: () => t("form.taPath"),
}

const provision = useMutation({
  mutationFn: () => deviceIdsApi.provision(profile.value!),
  onSuccess: (data) => {
    result.value = data
    notifySuccess(t("result.done"))
  },
})

function submit() {
  if (profile.value?.dry_run) provision.mutate()
  else confirmOpen.value = true
}

const resultRows = computed(() => {
  const data = result.value
  if (!data) return []
  const none = t("result.notAvailable")
  return [
    { label: t("result.mode"), value: data.dry_run ? t("result.dry") : t("result.live") },
    { label: t("result.count"), value: data.count },
    {
      label: t("result.taVersion"),
      value: [data.ta_api_version, data.ta_version].filter(Boolean).join(" / ") || none,
    },
    { label: t("result.library"), value: data.loaded_library ?? none, mono: true },
    { label: t("result.report"), value: data.report_path, mono: true, copy: true },
    ...data.ids.map((id) => ({ label: id.label, value: id.value, mono: true })),
  ]
})
</script>

<template>
  <div class="flex flex-col gap-6 p-4">
    <QueryState
      :loading="defaults.isPending.value"
      :error="defaults.error.value"
      @retry="defaults.refetch()"
    >
      <template v-if="profile">
        <SectionCard :title="t('form.main')" :description="t('form.mainHint')">
          <template #action>
            <Button
              variant="ghost"
              size="icon"
              :aria-label="t('form.reload')"
              @click="defaults.refetch()"
            >
              <RefreshCw />
            </Button>
          </template>
          <div class="grid grid-cols-2 gap-x-3 gap-y-5">
            <Field v-for="key in mainFields" :key="key">
              <FieldLabel :for="`ids-${key}`">{{ labels[key]() }}</FieldLabel>
              <Input :id="`ids-${key}`" v-model="profile[key]" autocapitalize="off" />
            </Field>
          </div>
        </SectionCard>

        <Card class="py-0">
          <Collapsible v-model:open="advancedOpen">
            <CollapsibleTrigger
              class="state-layer focus-visible:ring-ring/50 relative flex min-h-18 w-full items-center justify-between gap-4 rounded-lg p-4 text-start outline-none focus-visible:ring-3"
            >
              <span class="flex flex-col gap-1">
                <span class="text-base">{{ t("form.advanced") }}</span>
                <span class="text-muted-foreground text-sm">{{ t("form.advancedHint") }}</span>
              </span>
              <ChevronDown
                :class="[
                  'text-muted-foreground size-5 shrink-0 transition-transform duration-350 ease-spring-fast',
                  advancedOpen ? 'rotate-180' : '',
                ]"
              />
            </CollapsibleTrigger>
            <CollapsibleContent>
              <CardContent class="grid grid-cols-2 gap-x-3 gap-y-5 px-4 pt-1 pb-4">
                <Field v-for="key in advancedFields" :key="key">
                  <FieldLabel :for="`ids-${key}`">{{ labels[key]() }}</FieldLabel>
                  <Input
                    :id="`ids-${key}`"
                    v-model="profile[key]"
                    class="font-mono text-sm"
                    autocapitalize="off"
                  />
                </Field>
              </CardContent>
            </CollapsibleContent>
          </Collapsible>
        </Card>

        <Field orientation="horizontal" class="bg-card gap-4 rounded-lg p-4">
          <FieldContent>
            <FieldLabel for="dry-run">{{ t("form.dryRun") }}</FieldLabel>
            <FieldDescription>{{ t("form.dryRunHint") }}</FieldDescription>
          </FieldContent>
          <Switch id="dry-run" v-model="profile.dry_run" />
        </Field>

        <Button
          size="lg"
          :variant="profile.dry_run ? 'default' : 'destructive'"
          :disabled="provision.isPending.value"
          @click="submit"
        >
          <Spinner v-if="provision.isPending.value" class="size-6" />
          <Send v-else />
          {{ profile.dry_run ? t("form.provisionDry") : t("form.provision") }}
        </Button>

        <SectionCard v-if="result" :title="t('result.title')">
          <KeyValueList :rows="resultRows" />
        </SectionCard>
      </template>
    </QueryState>

    <ConfirmDialog
      v-model:open="confirmOpen"
      :title="t('form.provision')"
      :description="t('form.confirm')"
      :confirm-label="t('form.provision')"
      destructive
      @confirm="provision.mutate()"
    />
  </div>
</template>
