import { describe, expect, it } from "vitest"

import { detectLocale, isRtl } from "./index"

const supported = ["en", "zh-CN", "zh-TW", "pt-BR", "es-ES", "ar", "de"]

describe("detectLocale", () => {
  it.each([
    [["zh-Hans-CN"], "zh-CN"],
    [["zh-Hant-HK"], "zh-TW"],
    [["zh-TW"], "zh-TW"],
    [["pt-PT"], "pt-BR"],
    [["es"], "es-ES"],
    [["de-AT"], "de"],
    [["ja", "ar-EG"], "ar"],
    [["ko"], "en"],
  ])("%j -> %s", (candidates, expected) => {
    expect(detectLocale(candidates, supported)).toBe(expected)
  })
})

describe("isRtl", () => {
  it("covers the right-to-left languages Tricky Addon ships", () => {
    expect(isRtl("ar")).toBe(true)
    expect(isRtl("fa")).toBe(true)
    expect(isRtl("zh-CN")).toBe(false)
  })
})
