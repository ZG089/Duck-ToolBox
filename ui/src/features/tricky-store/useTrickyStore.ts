import { computed, onMounted, reactive } from "vue"

import { bridgeStatus, copyToClipboard, packageLabels, pushToast } from "@/lib/api/client"
import type { BridgeStatus, Envelope } from "@/lib/api/client"
import { historyEntry } from "@/lib/bridge"
import * as api from "@/lib/api/tricky-store"
import { translate } from "@/i18n"
import type { CommandHistoryEntry } from "@/lib/types"

import type {
  DisplayPackage,
  KeyboxProvider,
  Policy,
  PolicySchema,
  TargetEntry,
  TargetMode,
  TrickyStoreStatus,
} from "./types"

export type PackageScope = "all" | "user" | "system"
type BusyKey = "status" | "save" | "keybox" | "props" | "apps" | "files" | "providers"

export interface TrickyStoreState {
  bridge: BridgeStatus
  busy: Record<BusyKey, boolean>
  history: CommandHistoryEntry[]
  status: TrickyStoreStatus | null
  schema: PolicySchema | null
  targets: Map<string, TargetMode>
  perAppPolicy: Record<string, Policy>
  defaultPolicy: Policy
  systemApps: string[]
  autoAddNewApps: boolean
  packages: DisplayPackage[]
  labels: Map<string, string>
  search: string
  scope: PackageScope
  providers: KeyboxProvider[]
  lastError: string
  errorDialogText: string
  errorDialogOpen: boolean
}

function iconUrl(packageName: string): string {
  return `ksu://icon/${packageName}`
}

export function useTrickyStore() {
  const state = reactive<TrickyStoreState>({
    bridge: bridgeStatus(),
    busy: { status: false, save: false, keybox: false, props: false, apps: false, files: false, providers: false },
    history: [],
    status: null,
    schema: null,
    targets: new Map(),
    perAppPolicy: {},
    defaultPolicy: {},
    systemApps: [],
    autoAddNewApps: false,
    packages: [],
    labels: new Map(),
    search: "",
    scope: "all",
    providers: [],
    lastError: "",
    errorDialogText: "",
    errorDialogOpen: false,
  })

  const backendName = computed(() => state.status?.active?.name ?? translate("trickyStore.noBackend"))
  const backendInstalled = computed(() => Boolean(state.status?.active))
  const selectedCount = computed(() => state.targets.size)

  const visiblePackages = computed(() => {
    const query = state.search.trim().toLowerCase()
    return state.packages.filter((entry) => {
      if (state.scope === "user" && entry.system) return false
      if (state.scope === "system" && !entry.system) return false
      if (!query) return true
      return `${entry.label} ${entry.package_name}`.toLowerCase().includes(query)
    })
  })

  function remember<T>(envelope: Envelope<T>): void {
    state.history.unshift(historyEntry(envelope))
    state.history.splice(10)
  }

  function accept<T>(envelope: Envelope<T>, successMessage?: string): T | null {
    remember(envelope)
    if (!envelope.ok || envelope.data === null) {
      reportEnvelope(envelope)
      return null
    }
    state.lastError = ""
    if (successMessage) {
      pushToast(successMessage)
    }
    return envelope.data
  }

  function reportEnvelope<T>(envelope: Envelope<T>): void {
    state.lastError = envelope.error?.message ?? translate("messages.commandFailed")
    const parts = [
      `${translate("dialog.command")}: ${envelope.command}`,
      `${translate("dialog.errorCode")}: ${envelope.error?.code ?? "unknown"}`,
      `${translate("dialog.errorMessage")}: ${state.lastError}`,
    ]
    if (envelope.error?.details !== undefined) {
      parts.push("", JSON.stringify(envelope.error.details, null, 2))
    }
    state.errorDialogText = parts.join("\n")
    state.errorDialogOpen = true
    pushToast(state.lastError)
  }

  async function withBusy(key: BusyKey, action: () => Promise<void>): Promise<void> {
    if (state.busy[key]) return
    state.busy[key] = true
    try {
      await action()
    } catch (error) {
      state.lastError = error instanceof Error ? error.message : translate("messages.unexpectedError")
      state.errorDialogText = state.lastError
      state.errorDialogOpen = true
      pushToast(state.lastError)
    } finally {
      state.busy[key] = false
    }
  }

  function applyStatus(status: TrickyStoreStatus): void {
    state.status = status
    state.schema = status.schema
    state.targets = new Map(status.config.targets.map((entry) => [entry.package_name, entry.mode]))
    state.perAppPolicy = { ...status.config.per_app_policy }
    state.defaultPolicy = { ...status.config.default_policy }
    state.systemApps = [...status.system_apps]
    state.autoAddNewApps = status.auto_add_new_apps
    state.packages = status.packages.map((entry) => ({
      ...entry,
      label: state.labels.get(entry.package_name) ?? entry.package_name,
      iconUrl: iconUrl(entry.package_name),
    }))
  }

  async function enrichLabels(): Promise<void> {
    const names = state.packages.map((entry) => entry.package_name)
    const labels = await packageLabels(names)
    if (labels.size === 0) return
    for (const [name, info] of labels) {
      state.labels.set(name, info.appLabel)
    }
    state.packages = state.packages.map((entry) => ({
      ...entry,
      label: state.labels.get(entry.package_name) ?? entry.label,
    }))
  }

  function syncPackageSelection(): void {
    state.packages = state.packages.map((entry) => ({
      ...entry,
      selected: state.targets.has(entry.package_name),
      mode: state.targets.get(entry.package_name) ?? "auto",
      tracked_system: state.systemApps.includes(entry.package_name),
    }))
  }

  function ensurePackage(packageName: string, system = false): void {
    if (state.packages.some((entry) => entry.package_name === packageName)) return
    state.packages.push({
      package_name: packageName,
      system,
      selected: false,
      mode: "auto",
      tracked_system: state.systemApps.includes(packageName),
      label: state.labels.get(packageName) ?? packageName,
      iconUrl: iconUrl(packageName),
    })
  }

  const actions = {
    async refresh(): Promise<void> {
      await withBusy("status", async () => {
        const status = accept(await api.statusCommand())
        if (!status) return
        applyStatus(status)
        await enrichLabels()
        if (status.config_error) {
          pushToast(translate("trickyStore.configReadError"))
        }
      })
    },
    async save(): Promise<void> {
      await withBusy("save", async () => {
        const request = {
          targets: [...state.targets].map(([package_name, mode]) => ({ package_name, mode }) as TargetEntry),
          default_policy: state.defaultPolicy,
          per_app_policy: state.perAppPolicy,
          system_apps: state.systemApps,
          auto_add_new_apps: state.autoAddNewApps,
        }
        if (accept(await api.saveCommand(request), translate("messages.trickyStoreTargetsSaved"))) {
          await actions.refresh()
        }
      })
    },
    setSearch(value: string): void {
      state.search = value
    },
    setScope(scope: PackageScope): void {
      state.scope = scope
    },
    togglePackage(packageName: string): void {
      if (state.targets.has(packageName)) {
        state.targets.delete(packageName)
      } else {
        state.targets.set(packageName, "auto")
      }
      syncPackageSelection()
    },
    setMode(packageName: string, mode: TargetMode): void {
      state.targets.set(packageName, mode)
      if (mode === "auto") {
        delete state.perAppPolicy[packageName]
      }
      syncPackageSelection()
    },
    setPerAppPolicy(packageName: string, policy: Policy | null): void {
      if (policy && Object.keys(policy).length > 0) {
        state.perAppPolicy[packageName] = policy
      } else {
        delete state.perAppPolicy[packageName]
      }
    },
    selectAllVisible(select: boolean): void {
      for (const entry of visiblePackages.value) {
        if (select) {
          if (!state.targets.has(entry.package_name)) {
            state.targets.set(entry.package_name, "auto")
          }
        } else {
          state.targets.delete(entry.package_name)
        }
      }
      syncPackageSelection()
    },
    applyPackageList(list: string[], select: boolean): void {
      for (const packageName of list) {
        ensurePackage(packageName)
        if (select) {
          if (!state.targets.has(packageName)) {
            state.targets.set(packageName, "auto")
          }
        } else {
          state.targets.delete(packageName)
        }
      }
      syncPackageSelection()
    },
    addSystemApp(packageName: string): void {
      const normalized = packageName.trim().replace(/[!?]+$/, "")
      if (!normalized || state.systemApps.includes(normalized)) return
      state.systemApps = [...state.systemApps, normalized].sort()
      ensurePackage(normalized, true)
      state.targets.set(normalized, state.targets.get(normalized) ?? "auto")
      syncPackageSelection()
    },
    removeSystemApp(packageName: string): void {
      state.systemApps = state.systemApps.filter((entry) => entry !== packageName)
      state.targets.delete(packageName)
      state.packages = state.packages.filter((entry) => entry.package_name !== packageName || entry.system)
      syncPackageSelection()
    },
    setAutoAddNewApps(value: boolean): void {
      state.autoAddNewApps = value
    },
    async loadApps(kind: "xposed" | "denylist" | "unnecessary", refresh = false): Promise<void> {
      await withBusy("apps", async () => {
        const envelope =
          kind === "xposed"
            ? await api.xposedCommand()
            : kind === "denylist"
              ? await api.denylistCommand()
              : await api.unnecessaryCommand(refresh)
        const payload = accept(envelope)
        if (!payload) return
        const select = kind !== "unnecessary"
        actions.applyPackageList(payload.packages, select)
        pushToast(translate("trickyStore.appsApplied", { count: String(payload.packages.length) }))
      })
    },
    async installKeybox(kind: "aosp" | "generate" | "fetch" | "local", options?: { path?: string; url?: string; decode?: string }): Promise<void> {
      await withBusy("keybox", async () => {
        const envelope =
          kind === "aosp"
            ? await api.keyboxSetAospCommand()
            : kind === "generate"
              ? await api.keyboxGenerateCommand()
              : kind === "fetch"
                ? await api.keyboxFetchCommand(options?.url ?? "", options?.decode ?? "")
                : await api.keyboxInstallCommand(options?.path ?? "")
        if (accept(envelope, translate("messages.trickyStoreKeyboxInstalled"))) {
          await actions.refresh()
        }
      })
    },
    async saveProps(propHandlerEnabled: boolean, bootHash: string | null): Promise<void> {
      await withBusy("props", async () => {
        if (accept(await api.propsSaveCommand(propHandlerEnabled, bootHash), translate("messages.propsSaved"))) {
          await actions.refresh()
        }
      })
    },
    async loadProviders(): Promise<void> {
      await withBusy("providers", async () => {
        const payload = accept(await api.providersListCommand())
        if (payload) state.providers = payload
      })
    },
    async saveProviders(providers: KeyboxProvider[]): Promise<void> {
      await withBusy("providers", async () => {
        const payload = accept(await api.providersSaveCommand(providers), translate("messages.providersSaved"))
        if (payload) state.providers = payload
      })
    },
    async resetProviders(): Promise<void> {
      await withBusy("providers", async () => {
        const payload = accept(await api.providersResetCommand())
        if (payload) state.providers = payload
      })
    },
    async copyText(value: string): Promise<void> {
      pushToast(translate((await copyToClipboard(value)) ? "messages.copied" : "messages.clipboardUnsupported"))
    },
    dismissErrorDialog(): void {
      state.errorDialogOpen = false
    },
  }

  onMounted(async () => {
    if (state.bridge.mode === "unavailable") {
      state.lastError = translate("messages.ksuUnavailable")
      return
    }
    await actions.refresh()
    await actions.loadProviders()
  })

  return { state, actions, visiblePackages, backendName, backendInstalled, selectedCount }
}

export type TrickyStoreController = ReturnType<typeof useTrickyStore>
