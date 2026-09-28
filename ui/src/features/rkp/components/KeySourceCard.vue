<script setup lang="ts">
import { computed } from "vue"

import { Field, FieldDescription, FieldError, FieldLabel } from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group"
import { Textarea } from "@/components/ui/textarea"
import SectionCard from "@/core/components/SectionCard.vue"

import type { Draft } from "../draft"
import { DEFAULT_KDF_LABEL, HEX_HW_KEY, HEX_SEED } from "../draft"
import { useRkpI18n } from "../i18n"

const draft = defineModel<Draft>({ required: true })
const { t } = useRkpI18n()

const modes = computed(() => [
  { value: "hw-key" as const, label: t("profile.hwKey"), hint: t("profile.hwKeyHint") },
  { value: "seed" as const, label: t("profile.seed"), hint: t("profile.seedHint") },
])
const seedInvalid = computed(
  () => draft.value.seed_hex.trim() !== "" && !HEX_SEED.test(draft.value.seed_hex.trim()),
)
const keyInvalid = computed(
  () => draft.value.hw_key_hex.trim() !== "" && !HEX_HW_KEY.test(draft.value.hw_key_hex.trim()),
)
</script>

<template>
  <SectionCard :title="t('profile.keySource')">
    <div class="flex flex-col gap-4">
      <RadioGroup v-model="draft.mode" class="gap-2">
        <Label
          v-for="mode in modes"
          :key="mode.value"
          :for="`mode-${mode.value}`"
          class="has-[[data-state=checked]]:border-primary has-[[data-state=checked]]:bg-primary/5 flex items-start gap-3 rounded-lg border p-3 font-normal"
        >
          <RadioGroupItem :id="`mode-${mode.value}`" :value="mode.value" class="mt-0.5" />
          <span class="flex flex-col gap-0.5">
            <span class="font-medium">{{ mode.label }}</span>
            <span class="text-muted-foreground text-sm">{{ mode.hint }}</span>
          </span>
        </Label>
      </RadioGroup>

      <Field v-if="draft.mode === 'seed'" :data-invalid="seedInvalid || undefined">
        <FieldLabel for="seed">{{ t("profile.seedValue") }}</FieldLabel>
        <Textarea
          id="seed"
          v-model="draft.seed_hex"
          class="font-mono text-xs"
          rows="3"
          autocapitalize="off"
          spellcheck="false"
          :placeholder="t('profile.seedPlaceholder')"
          :aria-invalid="seedInvalid"
        />
        <FieldError v-if="seedInvalid">{{ t("profile.seedPlaceholder") }}</FieldError>
      </Field>

      <template v-else>
        <Field :data-invalid="keyInvalid || undefined">
          <FieldLabel for="hw-key">{{ t("profile.hwKeyValue") }}</FieldLabel>
          <Input
            id="hw-key"
            v-model="draft.hw_key_hex"
            class="font-mono text-xs"
            autocapitalize="off"
            spellcheck="false"
            :placeholder="t('profile.hwKeyPlaceholder')"
            :aria-invalid="keyInvalid"
          />
          <FieldError v-if="keyInvalid">{{ t("profile.hwKeyPlaceholder") }}</FieldError>
        </Field>
        <Field>
          <FieldLabel for="kdf-label">{{ t("profile.kdfLabel") }}</FieldLabel>
          <Input id="kdf-label" v-model="draft.kdf_label" class="font-mono text-xs" />
          <FieldDescription>{{
            t("profile.kdfLabelHint", { label: DEFAULT_KDF_LABEL })
          }}</FieldDescription>
        </Field>
      </template>
    </div>
  </SectionCard>
</template>
