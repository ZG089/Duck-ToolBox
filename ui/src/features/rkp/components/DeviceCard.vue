<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query"
import { Smartphone } from "@lucide/vue"
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
import SectionCard from "@/core/components/SectionCard.vue"
import { notifySuccess } from "@/core/notify"

import type { Draft } from "../draft"
import { useRkpI18n } from "../i18n"
import { useRkpStore } from "../store"

const draft = defineModel<Draft>({ required: true })
const { t } = useRkpI18n()
const store = useRkpStore()

const detect = useMutation({
  mutationFn: store.detect,
  onSuccess: () => notifySuccess(t("profile.deviceLoaded")),
})

const numKeys = computed({
  get: () => String(draft.value.num_keys),
  set: (value: string) => (draft.value.num_keys = Number(value)),
})

const selects = computed(() => [
  {
    id: "curve",
    label: t("profile.curve"),
    options: [
      { value: "ed25519", label: t("choices.ed25519") },
      { value: "p256", label: t("choices.p256") },
    ],
  },
  {
    id: "security_level",
    label: t("profile.securityLevel"),
    options: [
      { value: "tee", label: t("choices.tee") },
      { value: "strongbox", label: t("choices.strongbox") },
    ],
  },
  {
    id: "vb_state",
    label: t("profile.vbState"),
    options: ["green", "yellow", "orange"].map((value) => ({
      value,
      label: t(`choices.${value as "green" | "yellow" | "orange"}`),
    })),
  },
  {
    id: "bootloader_state",
    label: t("profile.bootloader"),
    options: [
      { value: "locked", label: t("choices.locked") },
      { value: "unlocked", label: t("choices.unlocked") },
    ],
  },
])

function value(id: string): string {
  return id === "curve" ? draft.value.curve : String(draft.value.device[id as "vb_state"])
}

function update(id: string, next: unknown) {
  if (typeof next !== "string") return
  if (id === "curve") draft.value.curve = next as Draft["curve"]
  else draft.value.device[id as "vb_state"] = next
}
</script>

<template>
  <SectionCard :title="t('profile.device')">
    <template #action>
      <Button
        variant="outline"
        size="sm"
        :disabled="detect.isPending.value"
        @click="detect.mutate()"
      >
        <Spinner v-if="detect.isPending.value" />
        <Smartphone v-else />
        {{ t("profile.readDevice") }}
      </Button>
    </template>
    <div class="flex flex-col gap-4">
      <div class="bg-muted/50 rounded-lg p-3 text-sm">
        <p class="font-medium">{{ draft.device.brand }} {{ draft.device.model }}</p>
        <p class="text-muted-foreground font-mono text-xs">
          {{ draft.device.device }} · Android {{ draft.device.os_version }} ·
          {{ draft.device.system_patch_level }}
        </p>
      </div>
      <div class="grid grid-cols-2 gap-3">
        <Field v-for="select in selects" :key="select.id">
          <FieldLabel :for="select.id">{{ select.label }}</FieldLabel>
          <Select :model-value="value(select.id)" @update:model-value="update(select.id, $event)">
            <SelectTrigger :id="select.id" class="w-full"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="option in select.options"
                :key="option.value"
                :value="option.value"
              >
                {{ option.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </Field>
        <Field>
          <FieldLabel for="num-keys">{{ t("profile.numKeys") }}</FieldLabel>
          <Select v-model="numKeys">
            <SelectTrigger id="num-keys" class="w-full"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem v-for="count in [1, 2, 3, 4]" :key="count" :value="String(count)">
                {{ count }}
              </SelectItem>
            </SelectContent>
          </Select>
        </Field>
      </div>
    </div>
  </SectionCard>
</template>
