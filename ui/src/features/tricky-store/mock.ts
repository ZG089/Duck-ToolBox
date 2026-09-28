import type { MockModule } from "@/core/mock"
import { MOCK_PACKAGES, MockFailure } from "@/core/mock"

import type { Provider, SaveRequest } from "./api"

/** `?backend=tee-simulator|oh-my-keymint|tricky-store-legacy|none` picks the mocked module. */
const backend = new URLSearchParams(location.search).get("backend") ?? "tricky-store"

const modules: Record<string, { id: string; name: string; identity: string; version: string }> = {
  "tricky-store": {
    id: "tricky_store",
    name: "Tricky Store",
    identity: "TS",
    version: "v1.4.1",
  },
  "tricky-store-legacy": {
    id: "tricky_store",
    name: "Tricky Store",
    identity: "TS-L",
    version: "v1.2.1",
  },
  "tee-simulator": { id: "teesim", name: "TEESimulator", identity: "TEES", version: "v3.1" },
  "oh-my-keymint": {
    id: "oh_my_keymint",
    name: "OhMyKeymint",
    identity: "OMK",
    version: "v0.9.0",
  },
}

const text = (key: string, label: string, extra: Record<string, unknown> = {}) => ({
  key,
  label,
  kind: "text",
  options: [],
  multiline: false,
  ...extra,
})

const schemas: Record<string, unknown> = {
  "tricky-store": {
    supports_app_mode: true,
    supports_per_app_policy: true,
    default_policy: [
      text("os_patch", "System Patch", {
        options: ["prop", "no"],
        placeholder: "YYYYMM",
        max_length: 6,
        hint: "YYYYMM | prop | no",
      }),
      text("vendor_patch", "Vendor Patch", {
        options: ["prop", "no"],
        placeholder: "YYYYMMDD",
        max_length: 8,
        hint: "YYYYMMDD | prop | no",
      }),
      text("boot_patch", "Boot Patch", {
        options: ["prop", "no"],
        placeholder: "YYYYMMDD",
        max_length: 8,
        hint: "YYYYMMDD | prop | no",
      }),
    ],
  },
  "tee-simulator": {
    supports_app_mode: false,
    supports_per_app_policy: false,
    default_policy: [
      text("mode", "Operation Mode", { options: ["patch", "generation"], placeholder: "patch" }),
      text("os_patch", "System Patch", {
        options: ["today", "system_property", "harvested", "no"],
        placeholder: "today",
      }),
      text("brand", "Brand"),
      {
        key: "autoIncludeNewApps",
        label: "Auto-include New Apps",
        kind: "boolean",
        options: [],
        multiline: false,
      },
    ],
  },
  "oh-my-keymint": {
    supports_app_mode: false,
    supports_per_app_policy: false,
    default_policy: [
      text("security_patch", "Security Patch", {
        options: ["auto", "latest"],
        placeholder: "YYYY-MM-DD",
        max_length: 10,
      }),
      text("vb_hash", "VB Hash", {
        options: ["auto", "random"],
        placeholder: "64 hex chars",
        max_length: 64,
        multiline: true,
      }),
      {
        key: "device_locked",
        label: "Device Locked",
        kind: "boolean",
        options: [],
        multiline: false,
      },
    ],
  },
}
schemas["tricky-store-legacy"] = {
  ...(schemas["tricky-store"] as object),
  supports_app_mode: false,
  supports_per_app_policy: false,
}

let saved: SaveRequest = {
  targets: [
    { package_name: "com.google.android.gms", mode: "generate" },
    { package_name: "io.github.vvb2060.keyattestation", mode: "auto" },
    { package_name: "com.google.android.apps.walletnfcrel", mode: "hack" },
    { package_name: "com.example.removed.app", mode: "auto" },
  ],
  default_policy: { os_patch: "prop", vendor_patch: "no" },
  per_app_policy: { "com.google.android.gms": { os_patch: "202609" } },
  system_apps: ["com.google.android.gms", "com.android.vending"],
  auto_add_new_apps: false,
}
let props = { prop_handler_enabled: true, boot_hash: null as string | null }
let keyboxModified = 1_790_000_000
let entryEnabled = true
let providers: Provider[] = [
  { name: "Example provider", url: "https://example.com/keybox.txt", decode: "base64 -d" },
]

function active() {
  const module = modules[backend]
  if (!module) return null
  return {
    backend,
    identity: module.identity,
    module_id: module.id,
    module_dir: `/data/adb/modules/${module.id}`,
    name: module.name,
    version: module.version,
    version_code: 100,
    active: true,
  }
}

function requireBackend() {
  if (!active()) throw new MockFailure("no_backend", "no supported keystore module is installed")
}

function installed(source: string) {
  requireBackend()
  keyboxModified = Math.floor(Date.now() / 1000)
  return {
    backend,
    target_path: "/data/adb/tricky_store/keybox.xml",
    backup_path: "/data/adb/tricky_store/keybox.xml.bak",
    size: 4096,
    source,
    summary: { keyboxes: 1, has_ecdsa: true, has_rsa: true, chain_lengths: [3, 3] },
  }
}

const mock: MockModule = {
  feature: { id: "tricky-store", contract: 1 },
  handlers: {
    "tricky-store.status": () => {
      const selected = new Map(saved.targets.map((target) => [target.package_name, target.mode]))
      return {
        backends: active()
          ? [
              active(),
              {
                ...active(),
                backend: "oh-my-keymint",
                module_id: "oh_my_keymint",
                name: "OhMyKeymint",
                active: false,
              },
            ]
          : [],
        active: active(),
        schema: schemas[backend] ?? null,
        config: saved,
        config_error: null,
        keybox: active()
          ? {
              path: "/data/adb/tricky_store/keybox.xml",
              exists: true,
              size: 4096,
              modified_unix: keyboxModified,
            }
          : null,
        packages: MOCK_PACKAGES.map((entry) => ({
          package_name: entry.packageName,
          system: entry.system,
          selected: selected.has(entry.packageName),
          mode: selected.get(entry.packageName) ?? "auto",
          tracked_system: saved.system_apps.includes(entry.packageName),
        })),
        system_apps: saved.system_apps,
        auto_add_new_apps: saved.auto_add_new_apps,
        props,
        root_manager: { kind: new URLSearchParams(location.search).get("manager") ?? "kernel-su" },
      }
    },
    "tricky-store.save": (_args, _flags, input) => {
      requireBackend()
      const request = input as SaveRequest
      const patch = request.default_policy.os_patch
      if (patch && !/^(\d{6}|prop|no)$/.test(patch)) {
        throw new MockFailure(
          "invalid_policy",
          `invalid policy value for \`os_patch\`: expected YYYYMM, prop or no`,
        )
      }
      const restart_required =
        backend === "oh-my-keymint" &&
        request.default_policy.device_locked !== saved.default_policy.device_locked
      saved = request
      return {
        backend,
        target_count: request.targets.length,
        system_app_count: request.system_apps.length,
        auto_add_new_apps: request.auto_add_new_apps,
        restart_required,
      }
    },
    "tricky-store.keybox.install": (args) => installed(`local:${args[0]}`),
    "tricky-store.keybox.import": () => installed("local"),
    "tricky-store.keybox.set-aosp": () => installed("aosp"),
    "tricky-store.keybox.generate": () => installed("generated"),
    "tricky-store.keybox.fetch": (_args, flags) => {
      if (String(flags.url).includes("fail"))
        throw new MockFailure("download_failed", "download failed: HTTP 404")
      return installed("url")
    },
    "tricky-store.keybox.providers.list": () => providers,
    "tricky-store.keybox.providers.save": (_args, _flags, input) => {
      const next = input as Provider[]
      const bad = next.find(
        (entry) =>
          entry.decode &&
          !/^(cat|base64 -d|xxd -r -p)( \| (cat|base64 -d|xxd -r -p))*$/.test(entry.decode),
      )
      if (bad)
        throw new MockFailure("unsupported_decoder", `unsupported decode step \`${bad.decode}\``)
      providers = next
      return providers
    },
    "tricky-store.keybox.providers.reset": () => (providers = []),
    "tricky-store.keybox.providers.import": () => providers,
    "tricky-store.keybox.providers.import-content": () => providers,
    "tricky-store.keybox.providers.export": () => ({
      path: "/storage/emulated/0/Download/keybox-providers.json",
    }),
    "tricky-store.apps.denylist": () => ({
      packages: ["com.revolut.revolut", "com.paypal.android.p2pmobile"],
    }),
    "tricky-store.apps.xposed": () => ({ packages: ["com.example.xposed.hider"] }),
    "tricky-store.apps.unnecessary": () => ({
      packages: ["me.weishu.kernelsu", "org.lsposed.manager", "com.spotify.music"],
      source: "remote",
    }),
    "tricky-store.props": (_args, _flags, input) => {
      props = input as typeof props
      return {
        ...props,
        applied_boot_hash: !!props.boot_hash,
        synced_backend_policy: backend === "oh-my-keymint",
      }
    },
    "tricky-store.entry.status": () => ({
      enabled: entryEnabled,
      module_id: active()?.module_id ?? null,
      linked: entryEnabled && !!active(),
      has_own_webui: backend === "tee-simulator",
    }),
    "tricky-store.entry.enable": () => (
      (entryEnabled = true),
      mock.handlers["tricky-store.entry.status"]!([], {}, null)
    ),
    "tricky-store.entry.disable": () => (
      (entryEnabled = false),
      mock.handlers["tricky-store.entry.status"]!([], {}, null)
    ),
  },
}

export default mock
