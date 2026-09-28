import { useStorage } from "@vueuse/core"
import { computed } from "vue"
import type { I18nOptions } from "vue-i18n"
import { createI18n } from "vue-i18n"

import type { FeatureDefinition, LocaleLoaders } from "@/core/features"
import { features } from "@/core/features"

import en from "./locales/en.json"

export type { DotPath } from "./paths"
export { useFeatureI18n } from "./feature"

export const FALLBACK_LOCALE = "en"
const SYSTEM = "system"
const RTL_LANGUAGES = new Set(["ar", "fa", "he", "ur", "ps", "sd", "ku", "yi", "dv"])

const coreLoaders = import.meta.glob<{ default: Record<string, unknown> }>("./locales/*.json")

// Locales are discovered at runtime, so the locale type is a plain string.
export const i18n = createI18n<false, I18nOptions & { legacy: false; locale: string }>({
  legacy: false,
  locale: FALLBACK_LOCALE,
  fallbackLocale: FALLBACK_LOCALE,
  messages: { [FALLBACK_LOCALE]: { core: en } },
  missingWarn: false,
  fallbackWarn: false,
})

const preference = useStorage("duck-toolbox/locale", SYSTEM)

function localeOf(path: string): string {
  return path
    .split("/")
    .pop()!
    .replace(/\.json$/, "")
}

/**
 * Message loaders from `import.meta.glob("./locales/*.json")`, keyed by file name, so adding
 * a language is only adding its file.
 */
export function localeFiles(modules: Record<string, () => Promise<unknown>>): LocaleLoaders {
  return Object.fromEntries(
    Object.entries(modules).map(([path, load]) => [localeOf(path), load as LocaleLoaders[string]]),
  )
}

/** Locales that the shell or at least one feature ships messages for. */
export function availableLocales(): string[] {
  const locales = new Set(Object.keys(coreLoaders).map(localeOf))
  for (const feature of features()) {
    for (const locale of Object.keys(feature.messages)) locales.add(locale)
  }
  return [...locales].sort((left, right) => displayName(left).localeCompare(displayName(right)))
}

/** The language's own name for itself, e.g. `简体中文` for `zh-CN`. */
export function displayName(locale: string): string {
  try {
    return new Intl.DisplayNames([locale], { type: "language" }).of(locale) ?? locale
  } catch {
    return locale
  }
}

export function isRtl(locale: string): boolean {
  return RTL_LANGUAGES.has(locale.split("-")[0]!.toLowerCase())
}

/** Best supported match for the device languages, e.g. `zh-Hans-CN` -> `zh-CN`. */
export function detectLocale(candidates: readonly string[], supported: readonly string[]): string {
  const lower = new Map(supported.map((locale) => [locale.toLowerCase(), locale]))
  for (const candidate of candidates) {
    const tag = candidate.toLowerCase()
    const exact = lower.get(tag)
    if (exact) return exact
    const base = tag.split("-")[0]!
    const script = tag.includes("hant") || /-(tw|hk|mo)$/.test(tag) ? "zh-tw" : "zh-cn"
    const sameLanguage =
      (base === "zh" && lower.get(script)) ||
      lower.get(base) ||
      supported.find((locale) => locale.toLowerCase().startsWith(`${base}-`))
    if (sameLanguage) return sameLanguage
  }
  return FALLBACK_LOCALE
}

async function loadMessages(loaders: LocaleLoaders, locale: string) {
  const loader = loaders[locale]
  return loader ? (await loader()).default : undefined
}

async function loadLocale(locale: string, registered: readonly FeatureDefinition[]) {
  const bundle: Record<string, unknown> = {}
  const core = await loadMessages(
    Object.fromEntries(Object.entries(coreLoaders).map(([path, load]) => [localeOf(path), load])),
    locale,
  )
  if (core) bundle.core = core
  await Promise.all(
    registered.map(async (feature) => {
      const messages = await loadMessages(feature.messages, locale)
      if (messages) bundle[feature.namespace] = messages
    }),
  )
  i18n.global.mergeLocaleMessage(locale, bundle)
}

const loaded = new Set<string>()

/** Loads the locale (plus the English fallback) for the shell and every feature. */
export async function applyLocale(locale: string): Promise<void> {
  const registered = features()
  for (const needed of new Set([FALLBACK_LOCALE, locale])) {
    if (loaded.has(needed)) continue
    await loadLocale(needed, registered)
    loaded.add(needed)
  }
  i18n.global.locale.value = locale
  document.documentElement.lang = locale
  document.documentElement.dir = isRtl(locale) ? "rtl" : "ltr"
}

export function resolvedLocale(): string {
  const supported = availableLocales()
  if (preference.value !== SYSTEM && supported.includes(preference.value)) {
    return preference.value
  }
  return detectLocale(navigator.languages ?? [navigator.language], supported)
}

/** Language preference: `system` or an explicit locale. */
export function useLocalePreference() {
  return computed({
    get: () => preference.value,
    set: (value: string) => {
      preference.value = value
      void applyLocale(resolvedLocale())
    },
  })
}
