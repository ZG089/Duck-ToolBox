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
  <!-- eslint-disable-next-line vue/no-v-html -- sanitized by DOMPurify above -->
  <div class="prose prose-sm dark:prose-invert max-w-none" @click="onClick" v-html="html" />
</template>
