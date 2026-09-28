import { exec, getPackagesInfo, isKsuWebui, listPackages, toast } from "kernelsu-alt"

import { translate } from "@/i18n"

/** JSON envelope contract shared with `duckd`. `api` guards against version drift. */
export interface Envelope<T> {
  ok: boolean
  api?: number
  command: string
  data: T | null
  error: JsonError | null
  ts?: number
}

export interface JsonError {
  code: string
  message: string
  details?: unknown
}

export interface BridgeStatus {
  mode: "kernelsu" | "unavailable"
  moduleRoot: string
  dataRoot: string
}

/** Envelope API version this WebUI is built against. */
export const SUPPORTED_API = 1

const MODULE_ROOT_FALLBACK = "/data/adb/modules/duck-toolbox"
const DATA_ROOT_FALLBACK = "/data/adb/duck-toolbox"

export function bridgeAvailable(): boolean {
  return isKsuWebui() && typeof exec === "function"
}

export function bridgeStatus(): BridgeStatus {
  const moduleRoot = resolveModuleRoot()
  return {
    mode: bridgeAvailable() ? "kernelsu" : "unavailable",
    moduleRoot,
    dataRoot: resolveDataRoot(moduleRoot),
  }
}

function resolveModuleRoot(): string {
  const scope = globalThis as { ksu?: { moduleInfo?: () => string } }
  try {
    const raw = scope.ksu?.moduleInfo?.()
    if (raw) {
      const info = JSON.parse(raw) as Record<string, unknown>
      const dir = info.moduleDir ?? info.modulePath ?? info.path
      if (typeof dir === "string" && dir) {
        return dir
      }
      const id = info.moduleId ?? info.id
      if (typeof id === "string" && id) {
        return `/data/adb/modules/${id}`
      }
    }
  } catch {
    // moduleInfo is optional; fall back to the well-known path.
  }
  return MODULE_ROOT_FALLBACK
}

function resolveDataRoot(moduleRoot: string): string {
  return moduleRoot.startsWith("/data/adb/modules") ? DATA_ROOT_FALLBACK : moduleRoot
}

function shellQuote(value: string): string {
  return `'${value.replaceAll("'", `'\\''`)}'`
}

function commandLine(args: string[], stdin?: string): string {
  const wrapper = shellQuote(`${resolveModuleRoot()}/bin/duckctl.sh`)
  const joined = args.map(shellQuote).join(" ")
  if (!stdin) {
    return `${wrapper} ${joined}`
  }
  const marker = `__DUCK_${Math.random().toString(36).slice(2).toUpperCase()}__`
  return `cat <<'${marker}' | ${wrapper} ${joined}\n${stdin}\n${marker}`
}

/** Extracts the last valid `{...}` JSON object from mixed stdout/stderr. */
function extractJson(stdout: string, stderr: string): string {
  const lines = `${stdout}\n${stderr}`
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
  for (let index = lines.length - 1; index >= 0; index -= 1) {
    const line = lines[index]
    const start = line.indexOf("{")
    const end = line.lastIndexOf("}")
    if (start !== -1 && end > start) {
      const candidate = line.slice(start, end + 1)
      try {
        JSON.parse(candidate)
        return candidate
      } catch {
        // Try an earlier line.
      }
    }
  }
  throw new Error(stderr.trim() || stdout.trim() || "backend returned no JSON")
}

function envelopeError<T>(command: string, code: string, message: string, details?: unknown): Envelope<T> {
  return { ok: false, command, data: null, error: { code, message, details }, ts: now() }
}

function now(): number {
  return Math.floor(Date.now() / 1000)
}

/** Runs a `duckd` subcommand and parses its envelope. Never throws. */
export async function execJson<T>(args: string[], payload?: unknown): Promise<Envelope<T>> {
  const command = args.filter((arg) => !arg.startsWith("--")).join(".")
  if (!bridgeAvailable()) {
    return envelopeError<T>(command, "ksu_unavailable", translate("messages.ksuUnavailable"))
  }

  const root = resolveModuleRoot()
  const stdin = payload === undefined ? undefined : JSON.stringify(payload)
  const shell = commandLine(args, stdin)
  const env = { DUCK_TOOLBOX_ROOT: root, DUCK_TOOLBOX_DATA_ROOT: resolveDataRoot(root) }

  let result: Awaited<ReturnType<typeof exec>>
  try {
    result = await exec(shell, { cwd: root, env })
  } catch (error) {
    return envelopeError<T>(command, "exec_failed", errorMessage(error))
  }

  let envelope: Envelope<T>
  try {
    envelope = JSON.parse(extractJson(result.stdout, result.stderr)) as Envelope<T>
  } catch (error) {
    return envelopeError<T>(command, "json_parse_error", errorMessage(error), {
      stdout: result.stdout,
      stderr: result.stderr,
    })
  }

  if (envelope.api !== undefined && envelope.api !== SUPPORTED_API) {
    return envelopeError<T>(command, "api_mismatch", translate("messages.apiMismatch"), {
      expected: SUPPORTED_API,
      received: envelope.api,
    })
  }
  return envelope
}

export function errorMessage(error: unknown): string {
  return error instanceof Error && error.message.trim()
    ? error.message
    : translate("messages.unexpectedError")
}

export function pushToast(message: string): void {
  try {
    toast(message)
  } catch {
    console.info(message)
  }
}

/** Reads system properties in one `getprop` call, returning a name→value map. */
export async function readProps(keys: string[]): Promise<Record<string, string>> {
  if (!bridgeAvailable()) {
    return {}
  }
  const script = keys.map((key) => `printf '%s=' ${shellQuote(key)}; getprop ${shellQuote(key)}`).join("; ")
  try {
    const root = resolveModuleRoot()
    const result = await exec(script, {
      cwd: root,
      env: { DUCK_TOOLBOX_ROOT: root, DUCK_TOOLBOX_DATA_ROOT: resolveDataRoot(root) },
    })
    if (result.errno !== 0) {
      return {}
    }
    return Object.fromEntries(
      result.stdout
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter(Boolean)
        .map((line) => {
          const index = line.indexOf("=")
          return [line.slice(0, index), line.slice(index + 1)] as const
        }),
    )
  } catch {
    return {}
  }
}

export interface PackageInfo {
  packageName: string
  appLabel: string
  isSystem: boolean
}

/** App labels for the given packages, via the KernelSU package-manager API. */
export async function packageLabels(packages: string[]): Promise<Map<string, PackageInfo>> {
  const labels = new Map<string, PackageInfo>()
  if (!bridgeAvailable() || packages.length === 0) {
    return labels
  }
  try {
    const info = await getPackagesInfo(packages)
    for (const entry of info) {
      if (entry && typeof entry.packageName === "string") {
        labels.set(entry.packageName, {
          packageName: entry.packageName,
          appLabel: entry.appLabel || entry.packageName,
          isSystem: Boolean(entry.isSystem),
        })
      }
    }
  } catch {
    // Older managers lack getPackagesInfo; the caller falls back to package names.
  }
  return labels
}

export async function listInstalledPackages(scope: "all" | "user" | "system" = "all"): Promise<string[]> {
  if (!bridgeAvailable()) {
    return []
  }
  try {
    return await listPackages(scope)
  } catch {
    return []
  }
}

export async function copyToClipboard(value: string): Promise<boolean> {
  try {
    const clipboard = globalThis.navigator?.clipboard
    if (!clipboard?.writeText) {
      return false
    }
    await clipboard.writeText(value)
    return true
  } catch {
    return false
  }
}
