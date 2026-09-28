import { execJson, readProps } from "@/lib/api/client"
import type { Envelope } from "@/lib/api/client"
import type {
  InfoData,
  KeyboxData,
  PathsInfo,
  ProfileData,
  ProfileEnvelopeData,
  ProvisionData,
  VerifyData,
} from "@/lib/types"
import { defaultProfile } from "@/lib/types"

export function profileShow(): Promise<Envelope<ProfileEnvelopeData>> {
  return execJson(["rkp", "profile", "show", "--json"])
}

export function profileSave(profile: ProfileData): Promise<Envelope<ProfileEnvelopeData>> {
  return execJson(["rkp", "profile", "save", "--stdin-json", "--json"], profile)
}

export function profileClear(): Promise<Envelope<{ cleared: boolean; paths: PathsInfo }>> {
  return execJson(["rkp", "profile", "clear", "--json"])
}

export function infoCommand(): Promise<Envelope<InfoData>> {
  return execJson(["rkp", "info", "--json"])
}

export function provisionCommand(): Promise<Envelope<ProvisionData>> {
  return execJson(["rkp", "provision", "--json"])
}

export function keyboxCommand(): Promise<Envelope<KeyboxData>> {
  return execJson(["rkp", "keybox", "--json"])
}

export function verifyCommand(path: string): Promise<Envelope<VerifyData>> {
  return execJson(["rkp", "verify", path.trim(), "--json"])
}

/**
 * Builds a profile seeded from the current device's build properties, mirroring what a
 * factory RKP request would report. Used to pre-fill the RKP workbench.
 */
export async function systemProfileDefaults(): Promise<ProfileData | null> {
  const props = await readProps([
    "ro.product.brand",
    "ro.product.model",
    "ro.product.device",
    "ro.product.name",
    "ro.product.manufacturer",
    "ro.build.fingerprint",
    "ro.build.version.release",
    "remote_provisioning.hostname",
    "ro.boot.verifiedbootstate",
    "ro.boot.vbmeta.device_state",
    "ro.boot.vbmeta.digest",
    "ro.boot.flash.locked",
    "ro.build.version.security_patch",
    "ro.vendor.build.security_patch",
    "ro.bootimage.build.version.security_patch",
  ])
  if (Object.keys(props).length === 0) {
    return null
  }

  const base = defaultProfile()
  const serverHost = props["remote_provisioning.hostname"]
    ?.trim()
    .replace(/^https?:\/\//, "")
    .replace(/\/v1\/?$/, "")
    .replace(/\/+$/, "")

  return {
    ...base,
    device: {
      ...base.device,
      brand: props["ro.product.brand"] || base.device.brand,
      model: props["ro.product.model"] || base.device.model,
      device: props["ro.product.device"] || base.device.device,
      product: props["ro.product.name"] || base.device.product,
      manufacturer: props["ro.product.manufacturer"] || base.device.manufacturer,
      os_version: props["ro.build.version.release"] || base.device.os_version,
      vb_state: props["ro.boot.verifiedbootstate"] || base.device.vb_state,
      bootloader_state: normalizeBootloaderState(props),
      boot_patch_level: patchLevelDay(props, "ro.bootimage.build.version.security_patch"),
      system_patch_level: patchLevelMonth(props, "ro.build.version.security_patch"),
      vendor_patch_level: patchLevelDay(props, "ro.vendor.build.security_patch"),
      vbmeta_digest: props["ro.boot.vbmeta.digest"] ?? "",
    },
    fingerprint: { value: props["ro.build.fingerprint"] || base.fingerprint.value },
    server_url: serverHost ? `https://${serverHost}/v1` : base.server_url,
  }
}

function normalizeBootloaderState(props: Record<string, string>): string {
  const state = props["ro.boot.vbmeta.device_state"]?.trim()
  if (state) {
    return state
  }
  if (props["ro.boot.flash.locked"] === "1") {
    return "locked"
  }
  if (props["ro.boot.flash.locked"] === "0") {
    return "unlocked"
  }
  return "locked"
}

function digits(props: Record<string, string>, key: string): string {
  return (props[key] ?? props["ro.build.version.security_patch"] ?? "").replaceAll("-", "")
}

function patchLevelDay(props: Record<string, string>, key: string): number {
  const value = digits(props, key)
  return value.length >= 8 ? Number(value.slice(0, 8)) || 0 : 0
}

function patchLevelMonth(props: Record<string, string>, key: string): number {
  const value = digits(props, key)
  return value.length >= 6 ? Number(value.slice(0, 6)) || 0 : 0
}
