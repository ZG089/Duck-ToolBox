import { afterEach, describe, expect, it, vi } from "vitest"

import bundled from "./tricky-addon.json"
import {
  parseStrings,
  TRANSLATIONS_SOURCE,
  translationsVersion,
  updateTranslations,
  withDownloadedTranslations,
} from "./translation-update"
import { pick } from "./translations"

const xml = (entries: Record<string, string>) =>
  `<?xml version="1.0" encoding="utf-8"?><resources>${Object.entries(entries)
    .map(([name, value]) => `<string name="${name}">${value}</string>`)
    .join("")}</resources>`

function serve(files: Record<string, string>) {
  const fetch = vi.fn(async (url: string) => {
    const body = files[url.replace(`${TRANSLATIONS_SOURCE}/`, "")]
    return new Response(body ?? "missing", { status: body === undefined ? 404 : 200 })
  })
  vi.stubGlobal("fetch", fetch)
  return fetch
}

afterEach(() => {
  vi.unstubAllGlobals()
  localStorage.clear()
})

describe("strings.xml", () => {
  it("reads every string like Tricky Addon's loader", () => {
    const strings = parseStrings(xml({ menu_keybox: "Keybox", mode_auto: "Auto &amp; more" }))
    expect(strings.get("mode_auto")).toBe("Auto & more")
  })

  it("keeps only the strings this manager shows, converted", () => {
    const strings = new Map([
      ["prompt_keybox_repo_download_error", "Download failed for %s"],
      ["header_title", "Tricky Addon"],
      ["menu_keybox", "  "],
    ])
    expect(pick(strings)).toEqual({ prompt_keybox_repo_download_error: "Download failed for {0}" })
  })
})

describe("updateTranslations", () => {
  it("does nothing when upstream is not newer", async () => {
    const fetch = serve({ version: bundled.version })
    expect(await updateTranslations()).toBe("latest")
    expect(fetch).toHaveBeenCalledTimes(1)
  })

  it("downloads every language of a newer bundle", async () => {
    serve({
      version: "29991231",
      "languages.json": JSON.stringify({ en: "English", eo: "Esperanto" }),
      "strings/en.xml": xml({ menu_keybox: "Keyboxes" }),
      "strings/eo.xml": xml({ menu_keybox: "Ŝlosilujo" }),
    })
    expect(await updateTranslations()).toBe("updated")
    expect(translationsVersion()).toBe("29991231")

    const loaders = withDownloadedTranslations({
      en: async () => ({ default: { meta: { title: "Tricky Store" }, ta: { mode_auto: "Auto" } } }),
    })
    expect((await loaders.en!()).default).toEqual({
      meta: { title: "Tricky Store" },
      ta: { mode_auto: "Auto", menu_keybox: "Keyboxes" },
    })
    // A language only the download has becomes available.
    expect((await loaders.eo!()).default).toEqual({ ta: { menu_keybox: "Ŝlosilujo" } })
  })

  it("keeps the bundled strings when a download fails", async () => {
    serve({ version: "29991231", "languages.json": JSON.stringify({ en: "English" }) })
    await expect(updateTranslations()).rejects.toThrow("strings/en.xml: HTTP 404")
    expect(translationsVersion()).toBe(bundled.version)
  })

  it("rejects an unexpected version file", async () => {
    serve({ version: "<html>moved</html>" })
    await expect(updateTranslations()).rejects.toThrow("unexpected translation version")
  })
})
