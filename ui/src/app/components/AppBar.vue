<script setup lang="ts">
import { ArrowLeft, Settings } from "@lucide/vue"
import { useWindowScroll } from "@vueuse/core"
import { computed } from "vue"
import { useI18n } from "vue-i18n"
import { useRoute, useRouter } from "vue-router"

import { Button } from "@/components/ui/button"

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const { y } = useWindowScroll()

const isHome = computed(() => route.name === "home")
const title = computed(() => {
  const key = [...route.matched].reverse().find((record) => record.meta.titleKey)?.meta.titleKey
  return key ? t(key) : t("core.app.name")
})

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
  <header
    :class="[
      'bg-background/85 pt-safe px-safe sticky top-0 z-40 backdrop-blur transition-shadow',
      y > 4 ? 'shadow-sm' : '',
    ]"
  >
    <div class="mx-auto flex h-14 max-w-3xl items-center gap-1 px-2">
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
        class="ms-2 me-1 size-7 dark:invert"
        width="28"
        height="28"
      />
      <h1 class="min-w-0 flex-1 truncate px-1 text-lg font-semibold">{{ title }}</h1>
      <div id="app-bar-actions" class="flex items-center gap-0.5" />
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
