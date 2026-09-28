import { ShieldCheck } from "@lucide/vue"

import { defineFeature } from "@/core/features"
import { localeFiles } from "@/core/i18n"

import { trickyStoreApi } from "./api"
import EntrySettings from "./components/EntrySettings.vue"
import { NAMESPACE } from "./i18n"

const title = (key: string) => ({ titleKey: `${NAMESPACE}.${key}` })

export default defineFeature({
  id: "tricky-store",
  namespace: NAMESPACE,
  icon: ShieldCheck,
  order: 10,
  // Module ids of Tricky Store, OhMyKeymint and TEESimulator, whose WebUI entry may link here.
  hostModules: ["tricky_store", "oh_my_keymint", "teesim"],
  routes: [
    { path: "", name: "tricky-store", component: () => import("./pages/TargetsPage.vue") },
    {
      path: "keybox",
      component: () => import("./pages/KeyboxPage.vue"),
      meta: title("ta.menu_keybox"),
    },
    {
      path: "keybox/repo",
      component: () => import("./pages/RepoPage.vue"),
      meta: title("keybox.repoTitle"),
    },
    {
      path: "policy",
      component: () => import("./pages/PolicyPage.vue"),
      meta: title("ta.default_policy_title"),
    },
    {
      path: "props",
      component: () => import("./pages/PropsPage.vue"),
      meta: title("ta.menu_prop_setting"),
    },
    {
      path: "help",
      component: () => import("./pages/HelpPage.vue"),
      meta: title("ta.menu_help"),
    },
  ],
  messages: localeFiles(import.meta.glob("./locales/*.json")),
  contributes: {
    keyboxTargets: [
      {
        id: "tricky-store",
        labelKey: `${NAMESPACE}.meta.title`,
        async install(sourcePath) {
          const result = await trickyStoreApi.keyboxInstall(sourcePath)
          return { targetPath: result.target_path, backupPath: result.backup_path }
        },
      },
    ],
    settingsSections: [EntrySettings],
  },
})
