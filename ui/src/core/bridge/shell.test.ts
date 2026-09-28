import { describe, expect, it } from "vitest"

import { heredoc, shellQuote } from "./shell"

describe("shellQuote", () => {
  it("wraps plain words", () => {
    expect(shellQuote("tricky-store")).toBe("'tricky-store'")
  })

  it("keeps quotes and expansions literal", () => {
    expect(shellQuote(`a'b $(id) "c"`)).toBe(`'a'\\''b $(id) "c"'`)
  })
})

describe("heredoc", () => {
  it("never uses a delimiter that occurs in the payload", () => {
    const input = "DUCK_EOF_ABC\nline"
    const script = heredoc("cat", input)
    const marker = /<<'([A-Z0-9_]+)'/.exec(script)![1]!
    expect(input.includes(marker)).toBe(false)
    expect(script.endsWith(`\n${marker}`)).toBe(true)
  })
})
