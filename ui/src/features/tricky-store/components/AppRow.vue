<script setup lang="ts">
import { SlidersHorizontal } from "@lucide/vue"
import { onLongPress } from "@vueuse/core"
import { useTemplateRef } from "vue"

import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import AppIcon from "@/core/components/AppIcon.vue"

import type { TargetMode } from "../api"
import { useTrickyI18n } from "../i18n"

const props = defineProps<{
  packageName: string
  label: string
  system: boolean
  /** Mode when selected, `null` when the app is not a target. */
  mode: TargetMode | null
  customPolicy: boolean
  /** The backend supports per-app modes or policies. */
  tunable: boolean
}>()
const emit = defineEmits<{ toggle: []; tune: [] }>()
const { t } = useTrickyI18n()

const row = useTemplateRef<HTMLElement>("row")
let longPressed = false

// Tricky Addon opens the mode dialog on long press (context menu) of a selected app.
onLongPress(
  row,
  () => {
    if (props.mode === null || !props.tunable) return
    longPressed = true
    emit("tune")
  },
  { delay: 450, modifiers: { prevent: true } },
)

function click() {
  if (longPressed) {
    longPressed = false
    return
  }
  emit("toggle")
}

const markers: Record<TargetMode, string> = { auto: "", generate: "!", hack: "?" }
</script>

<template>
  <div
    ref="row"
    role="checkbox"
    tabindex="0"
    :aria-checked="mode !== null"
    :class="[
      'state-layer focus-visible:ring-ring/50 relative flex min-h-18 cursor-pointer items-center gap-4 py-3 ps-4 pe-3 transition-[border-radius,background-color] duration-350 ease-spring-fast outline-none select-none focus-visible:ring-3 [contain-intrinsic-size:auto_72px] [content-visibility:auto]',
      // Material 3 Expressive segmented list: a selected row turns fully round.
      mode !== null
        ? 'bg-secondary rounded-lg'
        : 'bg-card rounded-xs first:rounded-t-lg last:rounded-b-lg',
    ]"
    @click="click"
    @contextmenu.prevent
    @keydown.space.prevent="emit('toggle')"
    @keydown.enter.prevent="emit('toggle')"
  >
    <AppIcon :package-name="packageName" :label="label" />
    <div class="min-w-0 flex-1">
      <div class="truncate text-base">{{ label }}</div>
      <div dir="ltr" class="text-muted-foreground truncate font-mono text-xs rtl:text-right">
        {{ packageName }}
      </div>
      <div
        v-if="(mode && mode !== 'auto') || (customPolicy && mode !== null) || system"
        class="mt-1 flex flex-wrap gap-1"
      >
        <Badge v-if="mode && mode !== 'auto'">
          <span class="font-mono">{{ markers[mode] }}</span>
          {{ mode === "generate" ? t("ta.mode_certificate_generating") : t("ta.mode_leaf_hack") }}
        </Badge>
        <Badge v-if="customPolicy && mode !== null" variant="secondary" class="bg-card">{{
          t("list.customPolicy")
        }}</Badge>
        <Badge v-if="system" variant="outline">{{ t("list.system") }}</Badge>
      </div>
    </div>
    <Button
      v-if="mode !== null && tunable"
      variant="ghost"
      size="icon"
      :aria-label="t('list.tune', { name: label })"
      @click.stop="emit('tune')"
    >
      <SlidersHorizontal class="size-5" />
    </Button>
    <Checkbox
      :model-value="mode !== null"
      tabindex="-1"
      aria-hidden="true"
      class="pointer-events-none"
    />
  </div>
</template>
