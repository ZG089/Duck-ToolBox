<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query"
import { RefreshCw } from "@lucide/vue"
import { computed } from "vue"

import { Button } from "@/components/ui/button"
import { Field, FieldLabel } from "@/components/ui/field"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Spinner } from "@/components/ui/spinner"
import MarkdownView from "@/core/components/MarkdownView.vue"
import SectionCard from "@/core/components/SectionCard.vue"
import { notifySuccess } from "@/core/notify"

import { systemApi } from "../api"
import { useSystemI18n } from "../i18n"
import { updateChannel, useUpdateCheck } from "../queries"

const { t } = useSystemI18n()
const check = useUpdateCheck()
const channel = computed(() => (updateChannel.value === "canary" ? "canary" : "stable"))

const install = useMutation({
  mutationFn: () => systemApi.installUpdate(channel.value),
  onSuccess: () => notifySuccess(t("update.installed")),
})
const reboot = useMutation({ mutationFn: systemApi.reboot })

const status = computed(() => {
  const data = check.data.value
  if (!data) return ""
  return data.available
    ? t("update.available", { version: data.version ?? "" })
    : t("update.upToDate")
})
</script>

<template>
  <SectionCard id="update" :title="t('update.title')">
    <div class="flex flex-col gap-5">
      <Field>
        <FieldLabel for="update-channel">{{ t("update.channel") }}</FieldLabel>
        <Select v-model="updateChannel">
          <SelectTrigger id="update-channel" class="w-full"><SelectValue /></SelectTrigger>
          <SelectContent>
            <SelectItem value="stable">{{ t("update.stable") }}</SelectItem>
            <SelectItem value="canary">{{ t("update.canary") }}</SelectItem>
            <SelectItem value="off">{{ t("update.off") }}</SelectItem>
          </SelectContent>
        </Select>
      </Field>

      <div v-if="updateChannel !== 'off'" class="flex items-center justify-between gap-3">
        <p
          class="text-sm"
          :class="check.data.value?.available ? 'font-medium' : 'text-muted-foreground'"
        >
          {{ status }}
        </p>
        <Button
          variant="outline"
          size="sm"
          :disabled="check.isFetching.value"
          @click="check.refetch()"
        >
          <Spinner v-if="check.isFetching.value" />
          <RefreshCw v-else />
          {{ t("update.check") }}
        </Button>
      </div>

      <template v-if="check.data.value?.available">
        <div
          v-if="check.data.value.changelog"
          class="bg-muted max-h-72 overflow-y-auto rounded-md p-4"
        >
          <p class="mb-2 text-sm font-medium">{{ t("update.changelog") }}</p>
          <MarkdownView :source="check.data.value.changelog" />
        </div>
        <Button
          v-if="!install.isSuccess.value"
          size="lg"
          :disabled="install.isPending.value"
          @click="install.mutate()"
        >
          <Spinner v-if="install.isPending.value" class="size-6" />
          {{ t("update.install") }}
        </Button>
        <Button v-else size="lg" :disabled="reboot.isPending.value" @click="reboot.mutate()">
          {{ t("update.reboot") }}
        </Button>
      </template>
    </div>
  </SectionCard>
</template>
