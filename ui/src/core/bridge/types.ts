/** Result of a root shell command, as the KernelSU `exec` API reports it. */
export interface ExecResult {
  errno: number
  stdout: string
  stderr: string
}

export interface InstalledPackage {
  packageName: string
  label: string
  system: boolean
}

export interface HostModule {
  id: string
  /** Absolute module directory, e.g. `/data/adb/modules/duck-toolbox`. */
  dir: string
}

/**
 * What the WebUI needs from its host (KernelSU, APatch, WebUI X, ...). Everything else in
 * the app talks to the host only through this interface.
 */
export interface RootBridge {
  readonly available: boolean
  exec(command: string): Promise<ExecResult>
  toast(message: string): void
  /** The module whose WebUI is showing; may be another module linking to our webroot. */
  hostModule(): HostModule | null
  listPackages(kind: "user" | "system" | "all"): Promise<string[]>
  /** Labels for the given packages. Unknown packages are simply missing from the result. */
  packagesInfo(packageNames: string[]): Promise<InstalledPackage[]>
  /** URL that renders an app icon, or null when the host cannot serve icons. */
  iconUrl(packageName: string): string | null
  exit(): void
}
