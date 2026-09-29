<script setup lang="ts">
import "vue-sonner/style.css"

import { useWindowScroll } from "@vueuse/core"
import { ConfigProvider } from "reka-ui"
import { computed } from "vue"
import { useI18n } from "vue-i18n"
import { useRoute } from "vue-router"

import { Toaster } from "@/components/ui/sonner"
import ErrorDetailsDialog from "@/core/components/ErrorDetailsDialog.vue"
import { isRtl } from "@/core/i18n"
import { useDarkTheme } from "@/core/theme"

import AppBar from "./components/AppBar.vue"

const dark = useDarkTheme()
const route = useRoute()
const { t, locale } = useI18n()
const { y } = useWindowScroll()
// reka-ui lays its components out left to right unless told otherwise.
const dir = computed(() => (isRtl(locale.value) ? "rtl" : "ltr"))

const title = computed(() => {
  const key = [...route.matched].reverse().find((record) => record.meta.titleKey)?.meta.titleKey
  return key ? t(key) : t("core.app.name")
})
// Material 3 medium flexible app bar: the headline scrolls away and the bar takes the title.
const headline = computed(() => !route.matched.some((record) => record.meta.smallAppBar))
const collapsed = computed(() => !headline.value || y.value > 40)
</script>

<template>
  <ConfigProvider :dir="dir">
    <div class="min-h-dvh">
      <AppBar :title="title" :title-visible="collapsed" :title-is-heading="!headline" />
      <div v-if="headline" class="px-safe">
        <h1 class="mx-auto max-w-3xl px-4 pt-2 pb-2 text-[1.75rem] leading-9 break-words">
          {{ title }}
        </h1>
      </div>
      <main class="px-safe pb-safe mx-auto max-w-3xl">
        <RouterView v-slot="{ Component, route: current }">
          <component :is="Component" :key="current.matched[0]?.path" />
        </RouterView>
      </main>
      <Toaster
        :theme="dark ? 'dark' : 'light'"
        position="top-center"
        rich-colors
        close-button
        :offset="{ top: 'calc(var(--inset-top) + 16px)' }"
        :mobile-offset="{ top: 'calc(var(--inset-top) + 8px)' }"
        :toast-options="{ descriptionClass: 'whitespace-pre-line' }"
      />
      <ErrorDetailsDialog />
    </div>
  </ConfigProvider>
</template>
