<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query"
import { Copy, FileText } from "@lucide/vue"

import { Button } from "@/components/ui/button"
import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemMedia,
  ItemTitle,
} from "@/components/ui/item"
import QueryState from "@/core/components/QueryState.vue"
import SectionCard from "@/core/components/SectionCard.vue"
import { formatBytes, formatRelative } from "@/core/format"
import { useCopy } from "@/core/links"

import { systemApi } from "../api"
import { useSystemI18n } from "../i18n"

const { t } = useSystemI18n()
const copy = useCopy()
const artifacts = useQuery({ queryKey: ["system.artifacts"], queryFn: systemApi.artifacts })
</script>

<template>
  <SectionCard
    :title="t('files.title')"
    :description="
      artifacts.data.value
        ? t('files.folder', { path: artifacts.data.value.outputs_dir })
        : undefined
    "
  >
    <QueryState
      :loading="artifacts.isPending.value"
      :error="artifacts.error.value"
      :skeleton-rows="2"
      @retry="artifacts.refetch()"
    >
      <p v-if="!artifacts.data.value?.outputs.length" class="text-muted-foreground text-sm">
        {{ t("files.empty") }}
      </p>
      <div v-else class="-mx-2 flex flex-col">
        <Item v-for="file in artifacts.data.value.outputs.slice(0, 12)" :key="file.path" size="sm">
          <ItemMedia variant="icon"><FileText /></ItemMedia>
          <ItemContent class="min-w-0">
            <ItemTitle class="w-full truncate font-mono text-xs">{{ file.name }}</ItemTitle>
            <ItemDescription>
              {{ formatBytes(file.size) }} · {{ formatRelative(file.modified_unix) }}
            </ItemDescription>
          </ItemContent>
          <ItemActions>
            <Button size="icon" variant="ghost" class="size-8" @click="copy(file.path)">
              <Copy />
            </Button>
          </ItemActions>
        </Item>
      </div>
    </QueryState>
  </SectionCard>
</template>
