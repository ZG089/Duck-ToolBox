import { describe, expect, it } from "vitest"

import { repairMarkdown } from "./translations"

describe("repairMarkdown", () => {
  it("closes a link whose URL runs into the next word", () => {
    expect(
      repairMarkdown("从 [KeyAttestation](https://github.com/vvb2060/KeyAttestation/releases获取"),
    ).toBe("从 [KeyAttestation](https://github.com/vvb2060/KeyAttestation/releases)获取")
  })

  it("closes a code span that ends with a quote", () => {
    expect(
      repairMarkdown(
        "قيمة `verifiedBootHash' من [KeyAttestation](https://example.org). `ro.boot.vbmeta.digest`",
      ),
    ).toBe(
      "قيمة `verifiedBootHash` من [KeyAttestation](https://example.org). `ro.boot.vbmeta.digest`",
    )
  })

  it("leaves well-formed Markdown alone", () => {
    const text =
      "Get `verifiedBootHash` from [KeyAttestation](https://example.org/releases)获取, it's `x`'s."
    expect(repairMarkdown(text)).toBe(text)
  })
})
