// @vitest-environment node
import { afterEach, describe, expect, it } from "vitest"

import type { Host } from "@/core/testing/host"
import { DEVICE_PROPS, CONTRACT, hasDuckd, installHost } from "@/core/testing/host"

import { deviceIdsApi } from "./api"

let host: Host | undefined
afterEach(() => host?.dispose())

describe.skipIf(!hasDuckd)("Device ID API against the real duckd", CONTRACT, () => {
  it("reads defaults from the device and runs a dry run", async () => {
    host = installHost({ props: DEVICE_PROPS })
    const defaults = await deviceIdsApi.defaults()
    expect(defaults.brand).toBe("google")
    expect(defaults.serial).toBe("39021FDH2000AB")

    const result = await deviceIdsApi.provision({ ...defaults, dry_run: true })
    expect(result.dry_run).toBe(true)
    expect(result.loaded_library).toBeNull()
    expect(result.ids.map((id) => id.value)).toContain("google")
  })

  it("fills blank identifiers from the device, and refuses when the device has none", async () => {
    host = installHost({ props: DEVICE_PROPS })
    const defaults = await deviceIdsApi.defaults()
    const filled = await deviceIdsApi.provision({ ...defaults, brand: "", dry_run: true })
    expect(filled.ids.find((id) => id.label === "BRAND")?.value).toBe("google")
    host.dispose()

    const { "ro.product.brand": _brand, ...withoutBrand } = DEVICE_PROPS
    host = installHost({ props: withoutBrand })
    await expect(
      deviceIdsApi.provision({ ...defaults, brand: "", dry_run: true }),
    ).rejects.toMatchObject({ code: "missing_device_field" })
  })
})
