// @vitest-environment node
import { afterEach, describe, expect, it } from "vitest"

import { isDuckError } from "@/core/duckd"
import type { Host } from "@/core/testing/host"
import { DEVICE_PROPS, CONTRACT, hasDuckd, installHost } from "@/core/testing/host"

import type { Profile } from "./api"
import { rkpApi } from "./api"

let host: Host | undefined
afterEach(() => host?.dispose())

// Nothing listens here, so provisioning falls back to its local test mode.
const OFFLINE_SERVER = "http://127.0.0.1:9/v1"

async function seededProfile(): Promise<Profile> {
  const { profile } = await rkpApi.detect()
  return {
    ...profile,
    key_source: { kind: "seed", seed_hex: "11".repeat(32) },
    server_url: OFFLINE_SERVER,
  }
}

describe.skipIf(!hasDuckd)("RKP API against the real duckd", CONTRACT, () => {
  it("detects device values and keeps a saved profile", async () => {
    host = installHost({ props: DEVICE_PROPS })
    expect((await rkpApi.show()).profile.key_source.kind).toBe("unset")
    const detected = (await rkpApi.detect()).profile
    expect(detected.device.model).toBe("Pixel 9 Pro")
    expect(detected.device.vendor_patch_level).toBe(20260905)

    await rkpApi.save(await seededProfile())
    expect((await rkpApi.show()).profile.key_source).toEqual({
      kind: "seed",
      seed_hex: "11".repeat(32),
    })
  })

  it("derives the pinned Ed25519 key from a seed", async () => {
    host = installHost({ props: DEVICE_PROPS })
    await rkpApi.save(await seededProfile())
    // Same value as duck-rkp's seed_derivation_is_stable_across_crypto_upgrades fixture.
    expect((await rkpApi.info()).public_key_hex).toBe(
      "d04ab232742bb4ab3a1368bd4615e4e6d0224ab71a016baf8520a332c9778737",
    )
  })

  it("builds a CSR offline and verifies it", async () => {
    host = installHost({ props: DEVICE_PROPS })
    await rkpApi.save(await seededProfile())
    const provision = await rkpApi.provision()
    expect(provision.local_test_mode).toBe(true)
    expect(provision.local_verify.signature_valid).toBe(true)

    const verified = await rkpApi.verify(provision.csr_path)
    expect(verified.report.signature_valid).toBe(true)
    expect(verified.report.uds_pub_hex).toBe(provision.local_verify.uds_pub_hex)
  })

  it("reports a keybox request without a server as a backend error", async () => {
    host = installHost({ props: DEVICE_PROPS })
    await rkpApi.save(await seededProfile())
    const error = await rkpApi.keybox().catch((caught: unknown) => caught)
    expect(isDuckError(error)).toBe(true)
    expect(error).not.toMatchObject({ code: "contract_mismatch" })
  })
})
