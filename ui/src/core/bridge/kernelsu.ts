import { exec, getPackagesInfo, isKsuWebui, listPackages, toast } from "kernelsu-alt"

import type { HostModule, InstalledPackage, RootBridge } from "./types"

interface KsuGlobal {
  moduleInfo?: () => string
  exit?: () => void
}

function ksuGlobal(): KsuGlobal | undefined {
  return (globalThis as { ksu?: KsuGlobal }).ksu
}

/**
 * KernelSU WebUI host (also KernelSU Next, APatch and WebUI X, which expose the same `ksu`
 * object). Built on `kernelsu-alt`, a drop-in superset of the official `kernelsu` package
 * that falls back to `pm` when the manager has no package API.
 */
export function createKernelSuBridge(): RootBridge {
  return {
    available: true,

    exec(command) {
      // `env` and `cwd` are deliberately never used: the manager exports env values
      // unquoted (WebViewInterface.processOptions), so callers put quoted values in the
      // command itself.
      return exec(command)
    },

    toast(message) {
      toast(message)
    },

    hostModule(): HostModule | null {
      try {
        const raw = ksuGlobal()?.moduleInfo?.()
        if (!raw) return null
        const info = JSON.parse(raw) as { id?: unknown; moduleDir?: unknown }
        const dir = typeof info.moduleDir === "string" ? info.moduleDir : ""
        const id = typeof info.id === "string" ? info.id : dir.split("/").pop() || ""
        return dir || id ? { id, dir: dir || `/data/adb/modules/${id}` } : null
      } catch {
        return null
      }
    },

    async listPackages(kind) {
      try {
        return await listPackages(kind)
      } catch {
        return []
      }
    },

    async packagesInfo(packageNames) {
      if (packageNames.length === 0) return []
      try {
        const infos = await getPackagesInfo(packageNames)
        return (Array.isArray(infos) ? infos : [infos])
          .filter((info) => typeof info?.packageName === "string" && !("error" in info))
          .map<InstalledPackage>((info) => ({
            packageName: info.packageName,
            label: info.appLabel || info.packageName,
            system: Boolean(info.isSystem),
          }))
      } catch {
        return []
      }
    },

    iconUrl(packageName) {
      return `ksu://icon/${packageName}`
    },

    exit() {
      ksuGlobal()?.exit?.()
    },
  }
}

export function isKernelSuHost(): boolean {
  return isKsuWebui()
}
