import { describe, expect, it } from "vitest"

import type { FeatureDefinition } from "./types"
import { availability } from "./registry"

const feature = (fields: Partial<FeatureDefinition> = {}): FeatureDefinition => ({
  id: "tricky-store",
  namespace: "trickyStore",
  icon: {},
  order: 0,
  routes: [],
  messages: {},
  ...fields,
})

const manifest = {
  binary_version: "1.0.0",
  api: 1,
  features: [{ id: "tricky-store", summary: "", contract: 1 }],
}

describe("availability", () => {
  it("matches the backend feature and contract", () => {
    expect(availability(feature(), manifest)).toBe("available")
  })

  it("reports features the binary was built without", () => {
    expect(availability(feature({ id: "rkp" }), manifest)).toBe("missing")
  })

  it("disables a UI written against another contract", () => {
    expect(availability(feature({ contract: 2 }), manifest)).toBe("outdated")
  })

  it("never blocks UI-only features", () => {
    expect(availability(feature({ backend: null }), undefined)).toBe("available")
  })

  it("waits for the manifest", () => {
    expect(availability(feature(), undefined)).toBe("unknown")
  })
})
