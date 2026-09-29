<script setup lang="ts">
import DOMPurify from "dompurify"
import { marked } from "marked"
import { computed } from "vue"

import { openExternal } from "@/core/links"

/** Renders Markdown from translations or changelogs; remote HTML is sanitized. */
const props = defineProps<{ source: string }>()

const html = computed(() =>
  DOMPurify.sanitize(marked.parse(props.source, { async: false, gfm: true, breaks: true })),
)

function onClick(event: MouseEvent) {
  const anchor = (event.target as HTMLElement | null)?.closest("a")
  const href = anchor?.getAttribute("href")
  if (href && /^https?:\/\//.test(href)) {
    event.preventDefault()
    void openExternal(href)
  }
}
</script>

<template>
  <!-- eslint-disable vue/no-v-html -- sanitized by DOMPurify above -->
  <!-- Long URLs and code break where they would overflow; wide tables scroll on their own. -->
  <div
    class="prose prose-sm prose-neutral dark:prose-invert prose-table:block prose-table:overflow-x-auto max-w-none min-w-0 [overflow-wrap:anywhere]"
    @click="onClick"
    v-html="html"
  />
  <!-- eslint-enable vue/no-v-html -->
</template>
