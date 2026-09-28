<script setup lang="ts">
import "vue-sonner/style.css"

import { Toaster } from "@/components/ui/sonner"
import ErrorDetailsDialog from "@/core/components/ErrorDetailsDialog.vue"

import AppBar from "./components/AppBar.vue"
</script>

<template>
  <div class="min-h-dvh">
    <AppBar />
    <main class="px-safe pb-safe mx-auto max-w-3xl">
      <RouterView v-slot="{ Component, route }">
        <component :is="Component" :key="route.matched[0]?.path" />
      </RouterView>
    </main>
    <Toaster
      position="top-center"
      rich-colors
      close-button
      :offset="{ top: 'calc(var(--inset-top) + 16px)' }"
      :mobile-offset="{ top: 'calc(var(--inset-top) + 8px)' }"
      :toast-options="{ descriptionClass: 'whitespace-pre-line' }"
    />
    <ErrorDetailsDialog />
  </div>
</template>
