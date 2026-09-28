import { z } from "zod"

import { duckd } from "@/core/duckd"

const systemInfoSchema = z.object({
  module: z.object({
    id: z.string(),
    name: z.string(),
    version: z.string(),
    version_code: z.number(),
    author: z.string(),
    update_json: z.string().nullable(),
    binary_version: z.string(),
  }),
  root_manager: z.object({ kind: z.string(), version: z.string().nullable() }).nullable(),
  device: z.object({
    brand: z.string(),
    model: z.string(),
    device: z.string(),
    android_release: z.string(),
    sdk: z.string(),
    security_patch: z.string(),
    fingerprint: z.string(),
    abi: z.string(),
    kernel_release: z.string().nullable(),
    selinux: z.string().nullable(),
  }),
})
export type SystemInfo = z.infer<typeof systemInfoSchema>

export type UpdateChannel = "stable" | "canary"

const updateInfoSchema = z.object({
  channel: z.enum(["stable", "canary"]),
  current_version: z.string(),
  current_version_code: z.number(),
  available: z.boolean(),
  version: z.string().nullable(),
  version_code: z.number().nullable(),
  changelog: z.string().nullable(),
})
export type UpdateInfo = z.infer<typeof updateInfoSchema>

const installSchema = z.object({
  version: z.string(),
  version_code: z.number(),
  output: z.string(),
  reboot_required: z.boolean(),
})

const artifactsSchema = z.object({
  outputs: z.array(
    z.object({ name: z.string(), path: z.string(), size: z.number(), modified_unix: z.number() }),
  ),
  outputs_dir: z.string(),
  profile_path: z.string(),
  profile_secrets_path: z.string(),
  log_path: z.string(),
})
export type Artifacts = z.infer<typeof artifactsSchema>

const logSchema = z.object({
  entries: z.array(
    z.object({
      ts: z.number(),
      command: z.string(),
      ok: z.boolean(),
      code: z.string().optional(),
      message: z.string().optional(),
    }),
  ),
})
export type LogEntry = z.infer<typeof logSchema>["entries"][number]

export const systemApi = {
  info: () => duckd(["system", "info"], { schema: systemInfoSchema }),
  checkUpdate: (channel: UpdateChannel) =>
    duckd(["system", "update", "check", "--channel", channel], { schema: updateInfoSchema }),
  installUpdate: (channel: UpdateChannel) =>
    duckd(["system", "update", "install", "--channel", channel], { schema: installSchema }),
  artifacts: () => duckd(["system", "artifacts"], { schema: artifactsSchema }),
  log: (limit: number) => duckd(["system", "log", "--limit", String(limit)], { schema: logSchema }),
  reboot: () => duckd(["system", "reboot"]),
  uninstall: () => duckd(["system", "uninstall"]),
}
