import { z } from "zod"

import type { LocaleLoaders } from "@/core/features"

import bundled from "./tricky-addon.json"
import { pick } from "./translations"

/**
 * Tricky Addon's "Update translation bundle": newer translations are downloaded from its
 * repository and laid over the bundled `ta` messages, so fixes arrive without a module
 * update. Anything unexpected upstream only fails the download; the bundled strings stay.
 */
export const TRANSLATIONS_SOURCE =
  "https://raw.githubusercontent.com/KOWX712/Tricky-Addon-Update-Target-List/main/webui/public/locales"
const STORAGE_KEY = "duck-toolbox/tricky-addon-translations"

const downloadSchema = z.object({
  version: z.string().regex(/^\d+$/),
  messages: z.record(z.string(), z.record(z.string(), z.string())),
})
export type TranslationDownload = z.infer<typeof downloadSchema>

export function downloadedTranslations(): TranslationDownload | null {
  try {
    const parsed = downloadSchema.safeParse(JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "null"))
    return parsed.success ? parsed.data : null
  } catch {
    return null
  }
}

/** Version of the translations in use: the downloaded ones if any, else the bundled ones. */
export function translationsVersion(): string {
  const download = downloadedTranslations()
  return download && Number(download.version) > Number(bundled.version)
    ? download.version
    : bundled.version
}

/** The `<string name>` entries of an Android strings.xml, read like Tricky Addon's loader. */
export function parseStrings(xml: string): Map<string, string> {
  const document = new DOMParser().parseFromString(xml, "application/xml")
  if (document.getElementsByTagName("parsererror").length > 0) {
    throw new Error("not a strings.xml document")
  }
  return new Map(
    [...document.getElementsByTagName("string")].map((node) => [
      node.getAttribute("name") ?? "",
      node.textContent ?? "",
    ]),
  )
}

async function download(path: string): Promise<string> {
  const response = await fetch(`${TRANSLATIONS_SOURCE}/${path}`, { cache: "no-store" })
  if (!response.ok) throw new Error(`${path}: HTTP ${response.status}`)
  return response.text()
}

/** Downloads every language when upstream has a newer bundle than the one in use. */
export async function updateTranslations(): Promise<"updated" | "latest"> {
  const version = (await download("version")).trim()
  if (!/^\d+$/.test(version)) throw new Error(`unexpected translation version "${version}"`)
  if (Number(version) <= Number(translationsVersion())) return "latest"

  const languages = z
    .record(z.string().regex(/^[a-zA-Z-]+$/), z.string())
    .parse(JSON.parse(await download("languages.json")))
  const entries = await Promise.all(
    Object.keys(languages).map(
      async (code) => [code, pick(parseStrings(await download(`strings/${code}.xml`)))] as const,
    ),
  )
  const result: TranslationDownload = { version, messages: Object.fromEntries(entries) }
  localStorage.setItem(STORAGE_KEY, JSON.stringify(result))
  return "updated"
}

/**
 * Locale loaders with downloaded translations over the bundled `ta` group. Languages that
 * only the download has get loaders too, so they become selectable.
 */
export function withDownloadedTranslations(loaders: LocaleLoaders): LocaleLoaders {
  const download = downloadedTranslations()
  if (!download || Number(download.version) <= Number(bundled.version)) return loaders
  const merged: LocaleLoaders = { ...loaders }
  for (const [locale, ta] of Object.entries(download.messages)) {
    const load = loaders[locale]
    merged[locale] = async () => {
      const base = load ? (await load()).default : {}
      const bundledTa = (base.ta ?? {}) as Record<string, unknown>
      return { default: { ...base, ta: { ...bundledTa, ...ta } } }
    }
  }
  return merged
}
