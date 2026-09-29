<script setup lang="ts">
import { ChevronRight, KeyRound, TriangleAlert } from "@lucide/vue"
import { computed } from "vue"

import { Alert, AlertDescription } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Card } from "@/components/ui/card"

import type { BackendDetection, Status } from "../api"
import { useTrickyI18n } from "../i18n"

const props = defineProps<{ status: Status }>()
const { t, dynamic } = useTrickyI18n()

const name = (backend: BackendDetection) =>
  dynamic(`backend.names.${backend.backend}`, backend.name ?? backend.module_id)
const others = computed(() =>
  props.status.backends.filter((backend) => backend.module_id !== props.status.active?.module_id),
)
</script>

<template>
  <Card v-if="status.active" class="gap-3 p-4">
    <div class="min-w-0 px-1">
      <div class="flex min-w-0 items-center gap-2">
        <span class="truncate text-base font-medium">{{ name(status.active) }}</span>
        <Badge variant="secondary" class="shrink-0 font-mono">{{ status.active.identity }}</Badge>
      </div>
      <div class="text-muted-foreground truncate text-sm">
        {{ status.active.version }} ·
        <span class="font-mono text-xs">{{ status.active.module_id }}</span>
      </div>
    </div>

    <RouterLink
      to="/tricky-store/keybox"
      class="state-layer bg-muted focus-visible:ring-ring/50 relative flex min-h-12 items-center gap-3 rounded-md px-3 text-sm outline-none focus-visible:ring-3"
    >
      <KeyRound class="size-5 shrink-0" />
      <span :class="['flex-1', status.keybox?.exists ? '' : 'text-destructive font-medium']">
        {{ status.keybox?.exists ? t("keybox.status") : t("keybox.missing") }}
      </span>
      <ChevronRight class="rtl-flip text-muted-foreground size-5" />
    </RouterLink>

    <div v-if="others.length" class="flex flex-wrap gap-1.5">
      <Badge v-for="backend in others" :key="backend.module_id" variant="outline">
        {{ name(backend) }} · {{ t("backend.inactive") }}
      </Badge>
    </div>
    <Alert v-if="!status.active.active" class="bg-muted">
      <TriangleAlert />
      <AlertDescription>
        {{ t("backend.disabled", { name: name(status.active) }) }}
      </AlertDescription>
    </Alert>
    <Alert v-if="status.config_error" variant="destructive">
      <TriangleAlert />
      <AlertDescription class="break-all">
        {{ t("backend.configError", { error: status.config_error }) }}
      </AlertDescription>
    </Alert>
  </Card>
</template>
