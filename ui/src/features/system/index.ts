import { PackageCheck } from "@lucide/vue"

import { defineFeature } from "@/core/features"
import { localeFiles } from "@/core/i18n"

import { NAMESPACE } from "./i18n"
import DeviceWidget from "./widgets/DeviceWidget.vue"
import UpdateBanner from "./widgets/UpdateBanner.vue"

export default defineFeature({
  id: "system",
  namespace: NAMESPACE,
  icon: PackageCheck,
  order: 100,
  placement: "system",
  routes: [
    { path: "", name: "system", component: () => import("./pages/SystemPage.vue") },
    {
      path: "activity",
      name: "system.activity",
      component: () => import("./pages/ActivityPage.vue"),
      meta: { titleKey: `${NAMESPACE}.activity.title` },
    },
  ],
  messages: localeFiles(import.meta.glob("./locales/*.json")),
  contributes: {
    homeWidgets: [UpdateBanner, DeviceWidget],
  },
})
