<script setup lang="ts">
import { computed } from "vue"

import type { Policy, PolicyField } from "@/features/tricky-store/types"

const props = defineProps<{
  fields: PolicyField[]
  modelValue: Policy
}>()

const emit = defineEmits<{
  "update:modelValue": [policy: Policy]
}>()

const values = computed(() => props.modelValue)

function update(key: string, value: string) {
  const next: Policy = { ...props.modelValue }
  if (value.trim()) {
    next[key] = value
  } else {
    delete next[key]
  }
  emit("update:modelValue", next)
}
</script>

<template>
  <div class="policy-grid">
    <div v-for="field in fields" :key="field.key" class="field-group">
      <label class="field-label" :for="`policy-${field.key}`">{{ field.label }}</label>
      <textarea
        v-if="field.multiline"
        :id="`policy-${field.key}`"
        class="text-input"
        rows="2"
        :maxlength="field.max_length"
        :placeholder="field.placeholder"
        :value="values[field.key] ?? ''"
        @input="update(field.key, ($event.target as HTMLTextAreaElement).value)"
      />
      <input
        v-else
        :id="`policy-${field.key}`"
        class="text-input"
        autocapitalize="none"
        autocomplete="off"
        spellcheck="false"
        :maxlength="field.max_length"
        :placeholder="field.placeholder"
        :value="values[field.key] ?? ''"
        @input="update(field.key, ($event.target as HTMLInputElement).value)"
      >
      <div v-if="field.options?.length" class="chip-row">
        <button
          v-for="option in field.options"
          :key="option"
          class="option-chip"
          type="button"
          @click="update(field.key, option)"
        >
          {{ option }}
        </button>
      </div>
      <small v-if="field.hint" class="field-hint">{{ field.hint }}</small>
    </div>
  </div>
</template>
