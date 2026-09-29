<script setup lang="ts">
import { CalendarCheck } from "@lucide/vue"
import { computed } from "vue"

import { Button } from "@/components/ui/button"
import { Field, FieldContent, FieldDescription, FieldLabel } from "@/components/ui/field"
import { Input } from "@/components/ui/input"
import { Switch } from "@/components/ui/switch"
import { Textarea } from "@/components/ui/textarea"

import type { Policy, PolicyField } from "../api"
import { useTrickyI18n } from "../i18n"
import { todayValue } from "../selection"

/** Renders whatever policy fields the active backend's schema declares. */
const props = defineProps<{ fields: PolicyField[]; idPrefix: string }>()
const policy = defineModel<Policy>({ required: true })
const { t, dynamic } = useTrickyI18n()

const dated = computed(() => props.fields.filter((field) => todayValue(field) !== null))

function label(field: PolicyField) {
  return dynamic(`policy.fields.${field.key}`, field.label)
}

function set(key: string, value: string) {
  const next = { ...policy.value }
  if (value.trim()) next[key] = value.trim()
  else delete next[key]
  policy.value = next
}

function fillToday() {
  const next = { ...policy.value }
  for (const field of dated.value) next[field.key] = todayValue(field)!
  policy.value = next
}
</script>

<template>
  <div class="flex flex-col gap-5">
    <template v-for="field in fields" :key="field.key">
      <Field v-if="field.kind === 'boolean'" orientation="horizontal">
        <FieldContent>
          <FieldLabel :for="`${idPrefix}-${field.key}`">{{ label(field) }}</FieldLabel>
        </FieldContent>
        <Switch
          :id="`${idPrefix}-${field.key}`"
          :model-value="policy[field.key] === 'true'"
          @update:model-value="set(field.key, $event ? 'true' : 'false')"
        />
      </Field>
      <Field v-else>
        <FieldLabel :for="`${idPrefix}-${field.key}`">{{ label(field) }}</FieldLabel>
        <Textarea
          v-if="field.multiline"
          :id="`${idPrefix}-${field.key}`"
          :model-value="policy[field.key] ?? ''"
          :placeholder="field.placeholder"
          :maxlength="field.max_length"
          rows="2"
          class="font-mono text-sm"
          autocapitalize="off"
          spellcheck="false"
          @update:model-value="set(field.key, String($event))"
        />
        <Input
          v-else
          :id="`${idPrefix}-${field.key}`"
          :model-value="policy[field.key] ?? ''"
          :placeholder="field.placeholder"
          :maxlength="field.max_length"
          autocapitalize="off"
          spellcheck="false"
          @update:model-value="set(field.key, String($event))"
        />
        <div v-if="field.options.length" class="flex flex-wrap gap-2">
          <Button
            v-for="option in field.options"
            :key="option"
            size="sm"
            :variant="policy[field.key] === option ? 'default' : 'outline'"
            class="font-mono"
            @click="set(field.key, option)"
          >
            {{ option }}
          </Button>
        </div>
        <FieldDescription v-if="field.hint" class="font-mono text-xs">{{
          field.hint
        }}</FieldDescription>
      </Field>
    </template>

    <Button v-if="dated.length" variant="outline" @click="fillToday">
      <CalendarCheck />
      {{ t("ta.functional_button_today") }}
    </Button>
  </div>
</template>
