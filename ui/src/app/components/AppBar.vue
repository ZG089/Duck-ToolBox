<script setup lang="ts">
import { ArrowLeft, Settings } from "@lucide/vue"
import { computed } from "vue"
import { useI18n } from "vue-i18n"
import { useRoute, useRouter } from "vue-router"

import { Button } from "@/components/ui/button"

defineProps<{
  title: string
  titleVisible: boolean
  /** The page has no headline of its own, so the bar title is its heading. */
  titleIsHeading: boolean
}>()

const route = useRoute()
const router = useRouter()
const { t } = useI18n()

const isHome = computed(() => route.name === "home")

function back() {
  if (window.history.state?.back) {
    router.back()
    return
  }
  // Deep link without history: go to the parent route, or home.
  const parent = route.matched.at(-2)
  void router.replace(parent && parent.path !== route.path ? parent.path : "/")
}
</script>

<template>
  <!-- Material 3 small app bar: 64dp, 48dp touch targets, the title in title large. -->
  <header class="bg-background pt-safe px-safe sticky top-0 z-40">
    <div class="mx-auto flex h-16 max-w-3xl items-center gap-2 px-2">
      <Button
        v-if="!isHome"
        variant="ghost"
        size="icon"
        :aria-label="t('core.nav.back')"
        @click="back"
      >
        <ArrowLeft class="rtl-flip" />
      </Button>
      <img
        v-else
        src="/duck-logo.svg"
        alt=""
        class="ms-2 size-7 dark:invert"
        width="28"
        height="28"
      />
      <component
        :is="titleIsHeading ? 'h1' : 'span'"
        :aria-hidden="titleIsHeading ? undefined : 'true'"
        :class="[
          'min-w-0 flex-1 truncate px-1 text-[1.375rem] leading-7 transition-opacity duration-200',
          titleVisible ? 'opacity-100' : 'opacity-0',
        ]"
      >
        {{ title }}
      </component>
      <div id="app-bar-actions" class="flex items-center gap-2" />
      <Button
        v-if="isHome"
        variant="ghost"
        size="icon"
        :aria-label="t('core.nav.settings')"
        @click="router.push('/settings')"
      >
        <Settings />
      </Button>
    </div>
  </header>
</template>
