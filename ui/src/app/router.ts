import type { RouteRecordRaw } from "vue-router"
import { createRouter, createWebHashHistory } from "vue-router"

import { bridge } from "@/core/bridge"
import type { FeatureDefinition } from "@/core/features"

import HomePage from "./pages/HomePage.vue"

declare module "vue-router" {
  interface RouteMeta {
    /** i18n key of the app bar title; the deepest matched route with one wins. */
    titleKey?: string
    /** Title in the app bar only, without the headline above the page (full-height pages). */
    smallAppBar?: boolean
  }
}

export function createAppRouter(features: readonly FeatureDefinition[]) {
  const routes: RouteRecordRaw[] = [
    { path: "/", name: "home", component: HomePage, meta: { titleKey: "core.app.name" } },
    {
      path: "/settings",
      name: "settings",
      component: () => import("./pages/SettingsPage.vue"),
      meta: { titleKey: "core.nav.settings" },
    },
    ...features.map<RouteRecordRaw>((feature) => ({
      path: `/${feature.id}`,
      meta: { titleKey: `${feature.namespace}.meta.title` },
      children: feature.routes,
    })),
    {
      path: "/:pathMatch(.*)*",
      name: "not-found",
      component: () => import("./pages/NotFoundPage.vue"),
    },
  ]

  // Hash history: WebViewAssetLoader has no server-side fallback for deep paths.
  const router = createRouter({
    history: createWebHashHistory(),
    routes,
    scrollBehavior: (to, from, saved) => {
      if (saved) return saved
      if (to.hash) return { el: to.hash, top: 80 }
      return to.path === from.path ? false : { top: 0 }
    },
  })

  // Opened from a keystore module that links our webroot: go straight to its feature.
  const hostId = bridge().hostModule()?.id
  const landing = features.find((feature) => hostId && feature.hostModules?.includes(hostId))
  if (landing) {
    router.beforeEach((to, from) =>
      from.matched.length === 0 && to.name === "home" ? `/${landing.id}` : true,
    )
  }

  return router
}
