import { z } from "zod"

import { duckd } from "@/core/duckd"

export const modes = ["auto", "generate", "hack"] as const
export type TargetMode = (typeof modes)[number]

const backendSchema = z.object({
  backend: z.string(),
  identity: z.string(),
  module_id: z.string(),
  module_dir: z.string(),
  name: z.string().nullable(),
  version: z.string().nullable(),
  version_code: z.number().nullable(),
  active: z.boolean(),
})
export type BackendDetection = z.infer<typeof backendSchema>

const fieldSchema = z.object({
  key: z.string(),
  label: z.string(),
  kind: z.enum(["text", "boolean"]).catch("text"),
  placeholder: z.string().optional(),
  options: z.array(z.string()).default([]),
  max_length: z.number().optional(),
  multiline: z.boolean().default(false),
  hint: z.string().optional(),
})
export type PolicyField = z.infer<typeof fieldSchema>

const schemaSchema = z.object({
  supports_app_mode: z.boolean(),
  supports_per_app_policy: z.boolean(),
  default_policy: z.array(fieldSchema),
})
export type PolicySchema = z.infer<typeof schemaSchema>

const policySchema = z.record(z.string(), z.string())
export type Policy = z.infer<typeof policySchema>

const targetSchema = z.object({ package_name: z.string(), mode: z.enum(modes).catch("auto") })
export type Target = z.infer<typeof targetSchema>

const statusSchema = z.object({
  backends: z.array(backendSchema),
  active: backendSchema.nullable(),
  schema: schemaSchema.nullable(),
  config: z.object({
    targets: z.array(targetSchema),
    default_policy: policySchema,
    per_app_policy: z.record(z.string(), policySchema),
  }),
  config_error: z.string().nullable(),
  keybox: z
    .object({ path: z.string(), exists: z.boolean(), size: z.number(), modified_unix: z.number() })
    .nullable(),
  packages: z.array(
    z.object({
      package_name: z.string(),
      system: z.boolean(),
      selected: z.boolean(),
      mode: z.enum(modes).catch("auto"),
      tracked_system: z.boolean(),
    }),
  ),
  system_apps: z.array(z.string()),
  auto_add_new_apps: z.boolean(),
  props: z.object({ prop_handler_enabled: z.boolean(), boot_hash: z.string().nullable() }),
  root_manager: z.object({ kind: z.string() }).nullable(),
})
export type Status = z.infer<typeof statusSchema>
export type PackageEntry = Status["packages"][number]

const savedSchema = z.object({
  target_count: z.number(),
  /** A changed setting takes effect only after the keystore module restarts. */
  restart_required: z.boolean(),
})
export type Saved = z.infer<typeof savedSchema>

export interface SaveRequest {
  targets: Target[]
  default_policy: Policy
  per_app_policy: Record<string, Policy>
  system_apps: string[]
  auto_add_new_apps: boolean
}

const installSchema = z.object({
  backend: z.string(),
  target_path: z.string(),
  backup_path: z.string().nullable(),
  size: z.number(),
  source: z.string(),
  summary: z.object({
    keyboxes: z.number(),
    has_ecdsa: z.boolean(),
    has_rsa: z.boolean(),
    chain_lengths: z.array(z.number()),
  }),
})
export type KeyboxInstall = z.infer<typeof installSchema>

const providerSchema = z.object({ name: z.string(), url: z.string(), decode: z.string() })
export type Provider = z.infer<typeof providerSchema>
const providersSchema = z.array(providerSchema)

const packageListSchema = z.object({ packages: z.array(z.string()) })

const propsSchema = z.object({
  prop_handler_enabled: z.boolean(),
  boot_hash: z.string().nullable(),
  applied_boot_hash: z.boolean(),
  synced_backend_policy: z.boolean(),
})

const entrySchema = z.object({
  enabled: z.boolean(),
  module_id: z.string().nullable(),
  linked: z.boolean(),
  has_own_webui: z.boolean(),
})
export type EntryStatus = z.infer<typeof entrySchema>

const ts = (...args: string[]) => ["tricky-store", ...args]
const keybox = (...args: string[]) => ts("keybox", ...args)

export const trickyStoreApi = {
  status: () => duckd(ts("status"), { schema: statusSchema }),
  save: (request: SaveRequest) => duckd(ts("save"), { input: request, schema: savedSchema }),

  keyboxInstall: (path: string) => duckd(keybox("install", path), { schema: installSchema }),
  keyboxImport: (content: string) =>
    duckd(keybox("import"), { input: { content }, schema: installSchema }),
  keyboxAosp: () => duckd(keybox("set-aosp"), { schema: installSchema }),
  keyboxGenerate: () => duckd(keybox("generate"), { schema: installSchema }),
  keyboxFetch: (url: string, decode = "") =>
    duckd(keybox("fetch", "--url", url, "--decode", decode), { schema: installSchema }),

  providers: () => duckd(keybox("providers", "list"), { schema: providersSchema }),
  saveProviders: (providers: Provider[]) =>
    duckd(keybox("providers", "save"), { input: providers, schema: providersSchema }),
  resetProviders: () => duckd(keybox("providers", "reset"), { schema: providersSchema }),
  importProviders: (path: string) =>
    duckd(keybox("providers", "import", path), { schema: providersSchema }),
  importProvidersContent: (content: string) =>
    duckd(keybox("providers", "import-content"), { input: { content }, schema: providersSchema }),
  exportProviders: () =>
    duckd(keybox("providers", "export"), { schema: z.object({ path: z.string() }) }),

  denylist: () => duckd(ts("apps", "denylist"), { schema: packageListSchema }),
  xposed: () => duckd(ts("apps", "xposed"), { schema: packageListSchema }),
  unnecessary: () => duckd(ts("apps", "unnecessary", "--refresh"), { schema: packageListSchema }),

  saveProps: (request: { prop_handler_enabled: boolean; boot_hash: string | null }) =>
    duckd(ts("props"), { input: request, schema: propsSchema }),

  entry: () => duckd(ts("entry", "status"), { schema: entrySchema }),
  setEntry: (enabled: boolean) =>
    duckd(ts("entry", enabled ? "enable" : "disable"), { schema: entrySchema }),
}
