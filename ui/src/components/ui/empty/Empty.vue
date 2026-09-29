<script setup lang="ts">
import type { HTMLAttributes } from "vue"
import { cn } from "@/lib/utils"

const props = defineProps<{
  class?: HTMLAttributes["class"]
}>()
</script>

<template>
  <div
    data-slot="empty"
    :class="cn(
      'bg-card flex min-w-0 flex-1 flex-col items-center justify-center gap-6 text-balance rounded-lg px-6 py-10 text-center',
      props.class,
    )"
  >
    <slot />
  </div>
</template>
