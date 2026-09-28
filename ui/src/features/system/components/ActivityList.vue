<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query"
import { CircleCheck, CircleX } from "@lucide/vue"

import { Item, ItemContent, ItemDescription, ItemMedia, ItemTitle } from "@/components/ui/item"
import QueryState from "@/core/components/QueryState.vue"
import { formatRelative } from "@/core/format"

import { systemApi } from "../api"
import { useSystemI18n } from "../i18n"

const props = withDefaults(defineProps<{ limit?: number }>(), { limit: 8 })
const { t } = useSystemI18n()
const log = useQuery({
  queryKey: ["system.log", () => props.limit],
  queryFn: () => systemApi.log(props.limit),
  staleTime: 0,
})
</script>

<template>
  <QueryState
    :loading="log.isPending.value"
    :error="log.error.value"
    :skeleton-rows="3"
    @retry="log.refetch()"
  >
    <p v-if="!log.data.value?.entries.length" class="text-muted-foreground text-sm">
      {{ t("activity.empty") }}
    </p>
    <div v-else class="-mx-2 flex flex-col">
      <Item
        v-for="(entry, index) in log.data.value.entries"
        :key="`${entry.ts}-${index}`"
        size="sm"
      >
        <ItemMedia variant="icon" :class="entry.ok ? 'text-primary' : 'text-destructive'">
          <CircleCheck v-if="entry.ok" />
          <CircleX v-else />
        </ItemMedia>
        <ItemContent class="min-w-0">
          <ItemTitle class="w-full truncate font-mono text-xs">{{ entry.command }}</ItemTitle>
          <ItemDescription class="line-clamp-2">
            {{ formatRelative(entry.ts) }}
            <template v-if="!entry.ok">
              · {{ entry.code ?? t("activity.failed") }}: {{ entry.message }}</template
            >
          </ItemDescription>
        </ItemContent>
      </Item>
    </div>
  </QueryState>
</template>
