<script setup lang="ts">
import { Copy } from "@lucide/vue"
import { useI18n } from "vue-i18n"

import { Button } from "@/components/ui/button"
import { useCopy } from "@/core/links"

export interface KeyValueRow {
  label: string
  value: string | number | null | undefined
  mono?: boolean
  copy?: boolean
}

defineProps<{ rows: KeyValueRow[] }>()

const { t } = useI18n()
const copy = useCopy()
</script>

<template>
  <dl class="divide-border divide-y">
    <div
      v-for="row in rows"
      :key="row.label"
      class="flex items-start justify-between gap-3 py-2.5 first:pt-0 last:pb-0"
    >
      <dt class="text-muted-foreground shrink-0 text-sm">{{ row.label }}</dt>
      <dd class="flex min-w-0 items-start gap-1 text-end">
        <span
          :class="[
            'min-w-0 text-sm break-all',
            row.mono ? 'font-mono text-xs leading-5' : 'font-medium',
          ]"
        >
          {{ row.value === null || row.value === undefined || row.value === "" ? "—" : row.value }}
        </span>
        <Button
          v-if="row.copy && row.value"
          size="icon"
          variant="ghost"
          class="-my-1 size-7 shrink-0"
          :aria-label="t('core.actions.copy')"
          @click="copy(String(row.value))"
        >
          <Copy class="size-3.5" />
        </Button>
      </dd>
    </div>
  </dl>
</template>
