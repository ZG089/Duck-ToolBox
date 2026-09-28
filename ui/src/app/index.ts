import { VueQueryPlugin } from "@tanstack/vue-query"
import { useDebounceFn } from "@vueuse/core"
import { createPinia } from "pinia"
import type { App } from "vue"

import { bridge } from "@/core/bridge"
import { duckd } from "@/core/duckd"
import type { FeatureDefinition } from "@/core/features"
import { registerFeatures } from "@/core/features"
import { applyLocale, i18n, resolvedLocale } from "@/core/i18n"
import { notifyError } from "@/core/notify"
import { createQueryClient } from "@/core/query"
import { startTheme } from "@/core/theme"

import { createAppRouter } from "./router"

/** Every `src/features/<id>/index.ts`: adding a folder is all it takes to add a tool. */
function discoverFeatures(): FeatureDefinition[] {
  const modules = import.meta.glob<{ default: FeatureDefinition }>("../features/*/index.ts", {
    eager: true,
  })
  return Object.values(modules).map((module) => module.default)
}

export async function installApp(app: App): Promise<void> {
  const features = discoverFeatures()
  registerFeatures(features)

  // After any change, refresh the status line KernelSU shows for this module.
  const describe = useDebounceFn(() => {
    if (bridge().available) void duckd(["describe"]).catch(() => undefined)
  }, 1500)

  const queryClient = createQueryClient({
    onMutationSuccess: describe,
    onMutationError: notifyError,
  })

  app.use(createPinia())
  app.use(i18n)
  app.use(VueQueryPlugin, { queryClient })
  app.use(createAppRouter(features))

  startTheme()
  await applyLocale(resolvedLocale())
}
