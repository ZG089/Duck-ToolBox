import type { Policy, PolicyField, SaveRequest, Status, TargetMode } from "./api"

/** Everything the Save button writes, held as mutable collections while editing. */
export interface Draft {
  targets: Map<string, TargetMode>
  perApp: Map<string, Policy>
  defaultPolicy: Policy
  systemApps: Set<string>
  autoAdd: boolean
}

export interface AppItem {
  packageName: string
  label: string
  system: boolean
  /** Selected in the saved config; the list is ordered by this so rows do not jump on tap. */
  saved: boolean
}

export function draftFromStatus(status: Status): Draft {
  return {
    targets: new Map(status.config.targets.map((target) => [target.package_name, target.mode])),
    perApp: new Map(Object.entries(status.config.per_app_policy)),
    defaultPolicy: { ...status.config.default_policy },
    systemApps: new Set(status.system_apps),
    autoAdd: status.auto_add_new_apps,
  }
}

const byKey = <T>(entries: Iterable<[string, T]>) =>
  [...entries].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))

export function toSaveRequest(draft: Draft): SaveRequest {
  const targets = byKey(draft.targets).map(([package_name, mode]) => ({ package_name, mode }))
  return {
    targets,
    default_policy: Object.fromEntries(byKey(Object.entries(draft.defaultPolicy))),
    per_app_policy: Object.fromEntries(
      byKey(draft.perApp).filter(([name]) => draft.targets.has(name)),
    ),
    system_apps: [...draft.systemApps].sort(),
    auto_add_new_apps: draft.autoAdd,
  }
}

/** Stable fingerprint of a draft, for "unsaved changes" detection. */
export function draftKey(draft: Draft): string {
  return JSON.stringify(toSaveRequest(draft))
}

/**
 * Apps shown in the target list: user apps, system apps the user added and anything already
 * targeted. Selected apps come first, then by label, like Tricky Addon.
 */
export function listedApps(
  status: Status,
  draft: Draft,
  labels: ReadonlyMap<string, string>,
): AppItem[] {
  const saved = new Set(status.config.targets.map((target) => target.package_name))
  const known = new Map(status.packages.map((entry) => [entry.package_name, entry.system]))
  for (const name of draft.targets.keys()) if (!known.has(name)) known.set(name, false)

  const items: AppItem[] = []
  for (const [packageName, system] of known) {
    if (system && !draft.systemApps.has(packageName) && !draft.targets.has(packageName)) continue
    const label = labels.get(packageName) || packageName
    items.push({ packageName, label, system, saved: saved.has(packageName) })
  }
  return items.sort(
    (a, b) =>
      Number(b.saved) - Number(a.saved) ||
      a.label.localeCompare(b.label) ||
      a.packageName.localeCompare(b.packageName),
  )
}

export function matches(item: Pick<AppItem, "label" | "packageName">, query: string): boolean {
  const needle = query.trim().toLowerCase()
  if (!needle) return true
  return (
    item.label.toLowerCase().includes(needle) || item.packageName.toLowerCase().includes(needle)
  )
}

export function select(draft: Draft, packages: Iterable<string>): number {
  let changed = 0
  for (const name of packages) {
    if (draft.targets.has(name)) continue
    draft.targets.set(name, "auto")
    changed += 1
  }
  return changed
}

export function deselect(draft: Draft, packages: Iterable<string>): number {
  let changed = 0
  for (const name of packages) if (draft.targets.delete(name)) changed += 1
  return changed
}

const pad = (value: number) => String(value).padStart(2, "0")

/** Today's patch level in the format a field's placeholder asks for, if it is a date field. */
export function todayValue(
  field: Pick<PolicyField, "placeholder">,
  now = new Date(),
): string | null {
  const year = String(now.getFullYear())
  const month = pad(now.getMonth() + 1)
  const day = pad(now.getDate())
  switch (field.placeholder) {
    case "YYYYMM":
      return `${year}${month}`
    case "YYYYMMDD":
      return `${year}${month}${day}`
    case "YYYY-MM":
      return `${year}-${month}`
    case "YYYY-MM-DD":
      return `${year}-${month}-${day}`
    default:
      return null
  }
}
