export type { BridgeStatus, Envelope, JsonError } from "@/lib/api/client"

export type KeySource =
  | { kind: "unset" }
  | { kind: "seed"; seed_hex: string }
  | { kind: "hw-key"; hw_key_hex: string; kdf_label: string }

export type DiceCurve = "ed25519" | "p256"

export interface DeviceInfo {
  brand: string
  model: string
  device: string
  product: string
  manufacturer: string
  fused: number
  vb_state: string
  os_version: string
  security_level: string
  bootloader_state: string
  boot_patch_level: number
  system_patch_level: number
  vendor_patch_level: number
  vbmeta_digest?: string | null
  dice_issuer: string
  dice_subject: string
}

export interface FingerprintConfig {
  value: string
}

export interface ProfileData {
  key_source: KeySource
  curve: DiceCurve
  device: DeviceInfo
  fingerprint: FingerprintConfig
  server_url: string
  num_keys: number
  output_path: string
}

export interface PathsInfo {
  root: string
  data_root: string
  var_dir: string
  profile_path: string
  profile_secrets_path: string
  outputs_dir: string
  tmp_dir: string
  logs_dir: string
  log_path: string
  wrapper_path: string
  binary_path: string
}

export interface ProfileEnvelopeData {
  profile: ProfileData
  paths: PathsInfo
}

export interface InfoData {
  mode: string
  curve: DiceCurve
  seed_hex: string
  public_key_hex: string
  ed25519_pubkey_hex?: string
  device: DeviceInfo
  fingerprint: string
  server_url: string
  num_keys: number
  output_path: string
}

export interface ChainSummary {
  certificates: number
  subjects: string[]
}

export interface ProvisionChain {
  index: number
  path: string
  summary: ChainSummary
}

export interface VerifyReport {
  version: number
  dice_entries: number
  uds_pub_hex: string
  signature_valid: boolean
  csr_version: number
  cert_type: string
  brand?: string | null
  keys_to_sign: number
}

export interface ProvisionData {
  mode: string
  curve: DiceCurve
  cdi_leaf_pubkey_hex: string
  challenge_hex: string
  csr_path: string
  csr_len: number
  protected_data_len: number
  local_verify: VerifyReport
  cert_chains: ProvisionChain[]
  local_test_mode: boolean
  fetch_eek_error?: string | null
  server_submission_error?: string | null
}

export interface KeyboxData {
  mode: string
  cdi_leaf_pubkey_hex: string
  challenge_hex: string
  csr_path: string
  keybox_path: string
  keybox_xml: string
  device_id: string
  chain_summary: ChainSummary
}

export interface VerifyData {
  path: string
  report: VerifyReport
}

export interface DeviceIdsProfileData {
  brand: string
  device: string
  product: string
  serial: string
  manufacturer: string
  model: string
  imei: string
  imei2: string
  meid: string
  meid2: string
  ta_name: string
  ta_path: string
  dry_run: boolean
}

export interface DeviceIdsProvisionedId {
  label: string
  value: string
}

export interface DeviceIdsProvisionData {
  count: number
  ids: DeviceIdsProvisionedId[]
  dry_run: boolean
  ta_name: string
  ta_path: string
  loaded_library: string | null
  ta_api_version: string | null
  ta_version: string | null
  report_path: string
}

export interface ArtifactFile {
  name: string
  path: string
  size: number
  modified_unix: number
}

export interface ArtifactsData {
  outputs: ArtifactFile[]
  profile_path: string
  profile_secrets_path: string
  log_path: string
}

export interface CommandHistoryEntry {
  command: string
  ok: boolean
  at: string
  message: string
}

export function defaultProfile(): ProfileData {
  return {
    key_source: { kind: "unset" },
    curve: "ed25519",
    device: {
      brand: "generic",
      model: "default",
      device: "default",
      product: "default",
      manufacturer: "generic",
      fused: 1,
      vb_state: "green",
      os_version: "13",
      security_level: "tee",
      bootloader_state: "locked",
      boot_patch_level: 20250101,
      system_patch_level: 202501,
      vendor_patch_level: 20250101,
      vbmeta_digest: "",
      dice_issuer: "Android",
      dice_subject: "KeyMint",
    },
    fingerprint: {
      value: "generic/default/default:13/TP1A.220624.014/0:user/release-keys",
    },
    server_url: "https://remoteprovisioning.googleapis.com/v1",
    num_keys: 1,
    output_path: "var/outputs/keybox.xml",
  }
}
