<script setup lang="ts">
import { computed, ref, watch } from "vue"

import { bridge } from "@/core/bridge"

/** App icon served by the manager (`ksu://icon/<package>`), with an initial as fallback. */
const props = defineProps<{ packageName: string; label?: string }>()

const failed = ref(false)
watch(
  () => props.packageName,
  () => (failed.value = false),
)

const source = computed(() => bridge().iconUrl(props.packageName))
const initial = computed(() => (props.label || props.packageName).trim().charAt(0).toUpperCase())
</script>

<template>
  <span
    class="bg-muted text-muted-foreground relative inline-flex size-10 shrink-0 items-center justify-center overflow-hidden rounded-xl text-sm font-semibold"
  >
    <span aria-hidden="true">{{ initial }}</span>
    <!-- Native lazy loading: long lists only request icons that scroll into view. -->
    <img
      v-if="source && !failed"
      :src="source"
      alt=""
      loading="lazy"
      decoding="async"
      draggable="false"
      class="absolute inset-0 size-full object-cover"
      @error="failed = true"
    />
  </span>
</template>
