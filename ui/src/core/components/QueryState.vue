<script setup lang="ts">
import { CircleAlert } from "@lucide/vue"
import { useI18n } from "vue-i18n"

import { Button } from "@/components/ui/button"
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty"
import { Skeleton } from "@/components/ui/skeleton"
import { errorDescription, errorTitle, showErrorDetails } from "@/core/notify"

/** Loading skeleton, error with retry, or the default slot once data is present. */
defineProps<{
  loading: boolean
  error: unknown
  skeletonRows?: number
}>()

const emit = defineEmits<{ retry: [] }>()
const { t } = useI18n()
</script>

<template>
  <div v-if="loading" class="flex flex-col gap-3" aria-busy="true">
    <Skeleton v-for="row in skeletonRows ?? 3" :key="row" class="h-16 w-full rounded-lg" />
  </div>
  <Empty v-else-if="error">
    <EmptyHeader>
      <EmptyMedia variant="icon">
        <CircleAlert />
      </EmptyMedia>
      <EmptyTitle>{{ errorTitle(error) }}</EmptyTitle>
      <EmptyDescription class="line-clamp-4 break-words">
        {{ errorDescription(error) }}
      </EmptyDescription>
    </EmptyHeader>
    <EmptyContent class="flex-row justify-center gap-2">
      <Button variant="outline" @click="showErrorDetails(error)">
        {{ t("core.actions.details") }}
      </Button>
      <Button @click="emit('retry')">{{ t("core.actions.retry") }}</Button>
    </EmptyContent>
  </Empty>
  <slot v-else />
</template>
