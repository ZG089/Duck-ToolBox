/**
 * A stand-in for the KernelSU `window.ksu` object, used by `pnpm dev` in a desktop browser
 * and by the E2E build. It parses the exact shell lines `core/duckd` sends and answers
 * with envelopes from each feature's `mock.ts`, so the whole bridge path is exercised.
 *
 * Query parameters: `?host=<module id>` pretends the WebUI was opened from that module;
 * `?latency=<ms>` changes the simulated command time.
 */
import type { MockHandler, MockModule } from "@/core/mock"
import { MOCK_PACKAGES, MockFailure } from "@/core/mock"

const modules = Object.values(
  import.meta.glob<{ default: MockModule }>("../features/*/mock.ts", { eager: true }),
).map((module) => module.default)

const handlers = new Map<string, MockHandler>(
  modules.flatMap((module) => Object.entries(module.handlers)),
)

/** Splits the single-quoted words `core/bridge/shell.ts` produces. */
function words(line: string): string[] {
  return [...line.matchAll(/'((?:[^']|'\\'')*)'/g)].map((match) =>
    match[1]!.replaceAll(`'\\''`, "'"),
  )
}

function parse(command: string) {
  const [first = "", ...rest] = command.split("\n")
  const hasInput = /<<'(DUCK_EOF_[A-Z0-9]+)'$/.test(first)
  const input = hasInput ? JSON.parse(rest.slice(0, -1).join("\n")) : undefined
  const [, ...argv] = words(first)
  const positional: string[] = []
  const flags: Record<string, string | true> = {}
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index]!
    if (!arg.startsWith("--")) {
      positional.push(arg)
      continue
    }
    const next = argv[index + 1]
    if (
      next !== undefined &&
      !next.startsWith("--") &&
      arg !== "--stdin-json" &&
      arg !== "--json"
    ) {
      flags[arg.slice(2)] = next
      index += 1
    } else {
      flags[arg.slice(2)] = true
    }
  }
  return { positional, flags, input }
}

function builtin(label: string): unknown {
  if (label === "features") {
    return {
      binary_version: "0.1.0-dev",
      api: 1,
      features: modules.flatMap((module) =>
        module.feature
          ? [{ id: module.feature.id, summary: "mock", contract: module.feature.contract }]
          : [],
      ),
    }
  }
  if (label === "describe") return { description: "mock", applied: false }
  return undefined
}

export function respond(command: string): string {
  const { positional, flags, input } = parse(command)
  for (let length = positional.length; length > 0; length -= 1) {
    const label = positional.slice(0, length).join(".")
    const handler = handlers.get(label)
    const fixed = length === positional.length ? builtin(label) : undefined
    if (!handler && fixed === undefined) continue
    try {
      const data = handler ? handler(positional.slice(length), flags, input) : fixed
      return JSON.stringify({ ok: true, api: 1, command: label, data, error: null, ts: 0 })
    } catch (error) {
      const failure =
        error instanceof MockFailure ? error : new MockFailure("internal_error", String(error))
      return JSON.stringify({
        ok: false,
        api: 1,
        command: label,
        data: null,
        error: { code: failure.code, message: failure.message, details: failure.details },
        ts: 0,
      })
    }
  }
  return JSON.stringify({
    ok: false,
    api: 1,
    command: positional.join("."),
    data: null,
    error: { code: "usage_error", message: `mock has no handler for ${positional.join(" ")}` },
    ts: 0,
  })
}

export function installMockHost(): void {
  const scope = window as unknown as Record<string, unknown>
  if (scope.ksu) return
  const params = new URLSearchParams(location.search)
  const latency = Number(params.get("latency") ?? 150)
  const host = params.get("host") ?? "duck-toolbox"

  scope.ksu = {
    exec(command: string, _options: string, callback: string) {
      const stdout = respond(command)
      setTimeout(() => (scope[callback] as (...args: unknown[]) => void)?.(0, stdout, ""), latency)
    },
    toast: (message: string) => console.info("[toast]", message),
    moduleInfo: () => JSON.stringify({ id: host, moduleDir: `/data/adb/modules/${host}` }),
    listPackages: (kind: string) =>
      JSON.stringify(
        MOCK_PACKAGES.filter((pkg) => kind === "all" || pkg.system === (kind === "system")).map(
          (pkg) => pkg.packageName,
        ),
      ),
    getPackagesInfo: (json: string) =>
      JSON.stringify(
        (JSON.parse(json) as string[]).map((name) => {
          const pkg = MOCK_PACKAGES.find((candidate) => candidate.packageName === name)
          return pkg
            ? { packageName: name, appLabel: pkg.label, isSystem: pkg.system, versionName: "1.0" }
            : { packageName: name, error: "Package not found or inaccessible" }
        }),
      ),
    enableEdgeToEdge: () => {},
    exit: () => console.info("[exit]"),
  }
}
