<script setup lang="ts">
import { useEventListener, useIntervalFn } from "@vueuse/core"
import { computed, ref, useTemplateRef } from "vue"
import { useRouter } from "vue-router"
import { toast } from "vue-sonner"

import { Spinner } from "@/components/ui/spinner"

import { trickyStoreApi } from "../api"
import { useTrickyI18n } from "../i18n"
import { useKeyboxInstall } from "../keybox"

/** The site speaks the protocol in Tricky Addon's `keybox/repo/repo-api.md`. */
const REPO_ORIGIN = "https://keybox.kowx712.cc"

const { t, locale } = useTrickyI18n()
const router = useRouter()
const frame = useTemplateRef<HTMLIFrameElement>("frame")
const loading = ref(true)
const source = computed(() => `${REPO_ORIGIN}/${locale.value}`)

const install = useKeyboxInstall((url: string) => trickyStoreApi.keyboxFetch(url), {
  success: "ta.prompt_keybox_repo_set",
  failure: "ta.prompt_keybox_repo_set_error",
})

// Keep offering the handshake until the site acknowledges it.
const handshake = useIntervalFn(
  () => frame.value?.contentWindow?.postMessage({ type: "handshake" }, REPO_ORIGIN),
  500,
  { immediate: false, immediateCallback: true },
)

function loaded() {
  loading.value = false
  handshake.resume()
}

function leave() {
  handshake.pause()
  if (window.history.state?.back) router.back()
  else void router.replace("/tricky-store/keybox")
}

useEventListener(window, "message", (event: MessageEvent) => {
  if (event.origin !== REPO_ORIGIN || event.source !== frame.value?.contentWindow) return
  const message = event.data as { type?: string; url?: unknown; identity?: unknown }
  switch (message.type) {
    case "handshake_ack":
      handshake.pause()
      break
    case "download":
      if (typeof message.url !== "string") return
      install.mutate(message.url)
      leave()
      break
    case "error":
      toast.error(t("ta.prompt_keybox_repo_download_error", [String(message.identity ?? "")]))
      leave()
      break
  }
})
</script>

<template>
  <div class="relative h-[calc(100dvh-4rem-var(--inset-top))]">
    <div
      v-if="loading"
      class="text-muted-foreground absolute inset-0 flex flex-col items-center justify-center gap-3 text-sm"
    >
      <Spinner class="size-6" />
      {{ t("keybox.repoLoading") }}
    </div>
    <iframe
      ref="frame"
      :src="source"
      :title="t('keybox.repoTitle')"
      class="size-full border-0"
      allow="clipboard-read; clipboard-write"
      referrerpolicy="no-referrer"
      @load="loaded"
    />
  </div>
</template>
