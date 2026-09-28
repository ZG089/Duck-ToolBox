export type TargetMode = "auto" | "generate" | "hack"

export type Backend =
  | "tricky-store"
  | "tricky-store-legacy"
  | "tee-simulator"
  | "oh-my-keymint"

export interface BackendDetection {
  backend: Backend
  identity: string
  module_id: string
  module_dir: string
  name?: string | null
  version?: string | null
  version_code?: number | null
  active: boolean
}

export interface TargetEntry {
  package_name: string
  mode: TargetMode
}

export type Policy = Record<string, string>

export interface PolicyField {
  key: string
  label: string
  placeholder?: string
  options?: string[]
  max_length?: number
  multiline?: boolean
  hint?: string
}

export interface PolicySchema {
  supports_app_mode: boolean
  supports_per_app_policy: boolean
  default_policy: PolicyField[]
}

export interface ConfigData {
  targets: TargetEntry[]
  default_policy: Policy
  per_app_policy: Record<string, Policy>
}

export interface PackageEntry {
  package_name: string
  system: boolean
  selected: boolean
  mode: TargetMode
  tracked_system: boolean
}

export interface KeyboxStatus {
  path: string
  exists: boolean
  size: number
  modified_unix: number
}

export interface PropStatus {
  prop_handler_enabled: boolean
  boot_hash?: string | null
}

export interface RootManager {
  kind: "kernel-su" | "apatch" | "magisk"
  bin_dir: string
}

export interface TrickyStoreStatus {
  backends: BackendDetection[]
  active: BackendDetection | null
  schema: PolicySchema | null
  config: ConfigData
  config_error?: string | null
  keybox: KeyboxStatus | null
  packages: PackageEntry[]
  system_apps: string[]
  auto_add_new_apps: boolean
  props: PropStatus
  root_manager: RootManager | null
}

export interface SaveRequest {
  targets: TargetEntry[]
  default_policy: Policy
  per_app_policy: Record<string, Policy>
  system_apps: string[]
  auto_add_new_apps: boolean
}

export interface SaveResult {
  backend: Backend
  target_count: number
  system_app_count: number
  auto_add_new_apps: boolean
}

export interface KeyboxSummary {
  keyboxes: number
  has_ecdsa: boolean
  has_rsa: boolean
  chain_lengths: number[]
}

export interface KeyboxInstallResult {
  backend: Backend
  target_path: string
  backup_path?: string | null
  size: number
  source: string
  summary: KeyboxSummary
}

export interface KeyboxProvider {
  name: string
  url: string
  decode: string
}

export interface FileEntry {
  name: string
  path: string
  directory: boolean
  size: number
  modified_unix: number
}

export interface FileListData {
  path: string
  parent?: string | null
  entries: FileEntry[]
}

export interface PropSaveData {
  prop_handler_enabled: boolean
  boot_hash?: string | null
  applied_boot_hash: boolean
}

export interface PackageListData {
  packages: string[]
  source?: string
}

/** A target row enriched with the app label and icon resolved from KernelSU. */
export interface DisplayPackage extends PackageEntry {
  label: string
  iconUrl: string
}
