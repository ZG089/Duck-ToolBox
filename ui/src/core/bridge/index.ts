import { createKernelSuBridge, isKernelSuHost } from "./kernelsu"
import type { RootBridge } from "./types"

export type { ExecResult, HostModule, InstalledPackage, RootBridge } from "./types"
export { heredoc, shellQuote } from "./shell"

const unavailable: RootBridge = {
  available: false,
  exec: async () => ({ errno: 127, stdout: "", stderr: "no root WebUI host" }),
  toast: (message) => console.info(message),
  hostModule: () => null,
  listPackages: async () => [],
  packagesInfo: async () => [],
  iconUrl: () => null,
  exit: () => {},
}

let current: RootBridge | null = null

/** The host bridge, detected once. Browsers without a host get an inert bridge. */
export function bridge(): RootBridge {
  current ??= isKernelSuHost() ? createKernelSuBridge() : unavailable
  return current
}

/** Test seam: replaces the detected bridge. */
export function setBridge(next: RootBridge | null): void {
  current = next
}
