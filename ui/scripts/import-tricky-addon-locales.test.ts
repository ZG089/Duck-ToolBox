import { describe, expect, it } from "vitest"

import { convert } from "./import-tricky-addon-locales.mjs"

describe("convert", () => {
  it("turns Android format arguments into list interpolation", () => {
    expect(convert("Download failed for %s")).toBe("Download failed for {0}")
    expect(convert("%2$s after %1$s, 100%%")).toBe("{1} after {0}, 100%")
  })

  it("escapes vue-i18n message syntax", () => {
    expect(convert("a@b | {c}")).toBe("a{'@'}b {'|'} {'{'}c{'}'}")
  })

  it("unescapes Android string escapes", () => {
    expect(convert("l\\'app\\nnext")).toBe("l'app\nnext")
  })
})
