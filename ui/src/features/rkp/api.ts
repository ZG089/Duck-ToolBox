import { z } from "zod"

import { duckd } from "@/core/duckd"

const deviceSchema = z.object({
  brand: z.string(),
  model: z.string(),
  device: z.string(),
  product: z.string(),
  manufacturer: z.string(),
  fused: z.number(),
  vb_state: z.string(),
  os_version: z.string(),
  security_level: z.string(),
  bootloader_state: z.string(),
  boot_patch_level: z.number(),
  system_patch_level: z.number(),
  vendor_patch_level: z.number(),
  vbmeta_digest: z.string().nullable().optional(),
  dice_issuer: z.string(),
  dice_subject: z.string(),
})
export type DeviceInfo = z.infer<typeof deviceSchema>

const keySourceSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("unset") }),
  z.object({ kind: z.literal("seed"), seed_hex: z.string() }),
  z.object({ kind: z.literal("hw-key"), hw_key_hex: z.string(), kdf_label: z.string() }),
])

export const curves = ["ed25519", "p256"] as const
export type Curve = (typeof curves)[number]

const profileSchema = z.object({
  key_source: keySourceSchema,
  curve: z.enum(curves),
  device: deviceSchema,
  fingerprint: z.object({ value: z.string() }),
  server_url: z.string(),
  num_keys: z.number(),
  output_path: z.string(),
})
export type Profile = z.infer<typeof profileSchema>

const pathsSchema = z.object({
  profile_path: z.string(),
  profile_secrets_path: z.string(),
  outputs_dir: z.string(),
})

const profileEnvelopeSchema = z.object({ profile: profileSchema, paths: pathsSchema })
const detectSchema = z.object({ profile: profileSchema })

const infoSchema = z.object({
  mode: z.string(),
  curve: z.string(),
  seed_hex: z.string(),
  public_key_hex: z.string(),
  output_path: z.string(),
})
export type Info = z.infer<typeof infoSchema>

const chainSummarySchema = z.object({ certificates: z.number(), subjects: z.array(z.string()) })

const reportSchema = z.object({
  version: z.number(),
  dice_entries: z.number(),
  uds_pub_hex: z.string(),
  signature_valid: z.boolean(),
  csr_version: z.number(),
  cert_type: z.string(),
  brand: z.string().nullable().optional(),
  keys_to_sign: z.number(),
})
export type VerifyReport = z.infer<typeof reportSchema>

const provisionSchema = z.object({
  curve: z.string(),
  challenge_hex: z.string(),
  csr_path: z.string(),
  csr_len: z.number(),
  protected_data_len: z.number(),
  local_verify: reportSchema,
  cert_chains: z.array(
    z.object({ index: z.number(), path: z.string(), summary: chainSummarySchema }),
  ),
  local_test_mode: z.boolean(),
  fetch_eek_error: z.string().nullable().optional(),
  server_submission_error: z.string().nullable().optional(),
})
export type Provision = z.infer<typeof provisionSchema>

const keyboxSchema = z.object({
  csr_path: z.string(),
  keybox_path: z.string(),
  keybox_xml: z.string(),
  device_id: z.string(),
  chain_summary: chainSummarySchema,
})
export type Keybox = z.infer<typeof keyboxSchema>

const verifySchema = z.object({ path: z.string(), report: reportSchema })
export type Verify = z.infer<typeof verifySchema>

export const rkpApi = {
  show: () => duckd(["rkp", "profile", "show"], { schema: profileEnvelopeSchema }),
  save: (profile: Profile) =>
    duckd(["rkp", "profile", "save"], { input: profile, schema: profileEnvelopeSchema }),
  clear: () => duckd(["rkp", "profile", "clear"]),
  detect: () => duckd(["rkp", "profile", "detect"], { schema: detectSchema }),
  info: () => duckd(["rkp", "info"], { schema: infoSchema }),
  provision: () => duckd(["rkp", "provision"], { schema: provisionSchema }),
  keybox: () => duckd(["rkp", "keybox"], { schema: keyboxSchema }),
  verify: (path: string) => duckd(["rkp", "verify", path], { schema: verifySchema }),
}
