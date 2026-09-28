import type { Component } from "vue"
import type { RouteRecordRaw } from "vue-router"

/** Lazy loaders for a feature's message files, keyed by BCP 47 locale (`en`, `zh-CN`, ...). */
export type LocaleLoaders = Record<string, () => Promise<{ default: Record<string, unknown> }>>

export interface KeyboxInstallResult {
  targetPath: string
  backupPath: string | null
}

/** Somewhere a generated `keybox.xml` can be installed, e.g. the active keystore module. */
export interface KeyboxTarget {
  id: string
  /** Full i18n key of the target's name. */
  labelKey: string
  install(sourcePath: string): Promise<KeyboxInstallResult>
}

/** Extension points. Features plug into each other only through these. */
export interface Contributions {
  keyboxTargets?: KeyboxTarget[]
  /** Extra sections rendered on the Settings page, in feature order. */
  settingsSections?: Component[]
  /** Cards rendered on the home page above the tool list. */
  homeWidgets?: Component[]
}

export type ContributionKey = keyof Contributions
export type Contribution<K extends ContributionKey> = NonNullable<Contributions[K]>[number]

export interface FeatureDefinition {
  /** Route prefix, matching the backend feature id unless `backend` says otherwise. */
  id: string
  /** i18n namespace. `<namespace>.meta.title` and `.meta.description` name the feature. */
  namespace: string
  /** Backend feature this UI drives; `null` for UI-only features. Defaults to `id`. */
  backend?: string | null
  /** Backend contract version this UI was written against. Defaults to 1. */
  contract?: number
  icon: Component
  /** Sort order on the home page. */
  order: number
  /** `tool` and `system` features are listed on the home page; `hidden` only contribute. */
  placement?: "tool" | "system" | "hidden"
  /** Routes below `/<id>`. The route with an empty path is the landing page. */
  routes: RouteRecordRaw[]
  messages: LocaleLoaders
  /** Confirm `<namespace>.meta.warning` before opening the feature. */
  confirmOpen?: boolean
  /**
   * Modules that may link our webroot as their own WebUI. Opening the WebUI from one of
   * them lands directly on this feature.
   */
  hostModules?: string[]
  contributes?: Contributions
}

export function defineFeature(definition: FeatureDefinition): FeatureDefinition {
  return definition
}
