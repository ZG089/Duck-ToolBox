import { Fingerprint } from "@lucide/vue"

import { defineFeature } from "@/core/features"
import { localeFiles } from "@/core/i18n"

import { NAMESPACE } from "./i18n"

export default defineFeature({
  id: "device-ids",
  namespace: NAMESPACE,
  icon: Fingerprint,
  order: 30,
  confirmOpen: true,
  routes: [{ path: "", name: "device-ids", component: () => import("./pages/DeviceIdsPage.vue") }],
  messages: localeFiles(import.meta.glob("./locales/*.json")),
})
