import { describe, expect, it } from "vitest"

import type { Status } from "./api"
import {
  deselect,
  draftFromStatus,
  draftKey,
  listedApps,
  matches,
  select,
  todayValue,
  toSaveRequest,
} from "./selection"

const pkg = (package_name: string, system = false) => ({
  package_name,
  system,
  selected: false,
  mode: "auto" as const,
  tracked_system: false,
})

const status: Status = {
  backends: [],
  active: null,
  schema: null,
  config: {
    targets: [
      { package_name: "com.b.target", mode: "generate" },
      { package_name: "com.gone.app", mode: "auto" },
      { package_name: "com.android.vending", mode: "auto" },
    ],
    default_policy: { os_patch: "prop" },
    per_app_policy: {
      "com.b.target": { os_patch: "202609" },
      "com.not.targeted": { os_patch: "no" },
    },
  },
  config_error: null,
  keybox: null,
  packages: [
    pkg("com.a.user"),
    pkg("com.b.target"),
    pkg("com.c.user"),
    pkg("com.android.vending", true),
    pkg("com.android.settings", true),
    pkg("com.google.android.gms", true),
  ],
  system_apps: ["com.google.android.gms"],
  auto_add_new_apps: true,
  props: { prop_handler_enabled: true, boot_hash: null },
  root_manager: null,
}

const labels = new Map([
  ["com.a.user", "Zeta"],
  ["com.b.target", "Beta"],
  ["com.c.user", "Alpha"],
])

describe("save requests", () => {
  it("drops policies of apps that are no longer targets and sorts for stable output", () => {
    const request = toSaveRequest(draftFromStatus(status))
    expect(request.targets.map((target) => target.package_name)).toEqual([
      "com.android.vending",
      "com.b.target",
      "com.gone.app",
    ])
    expect(Object.keys(request.per_app_policy)).toEqual(["com.b.target"])
    expect(request.auto_add_new_apps).toBe(true)
  })

  it("detects changes by content, not identity", () => {
    const draft = draftFromStatus(status)
    const before = draftKey(draft)
    select(draft, ["com.a.user"])
    expect(draftKey(draft)).not.toBe(before)
    deselect(draft, ["com.a.user"])
    expect(draftKey(draft)).toBe(before)
  })
})

describe("listedApps", () => {
  it("lists user apps, tracked and targeted system apps and missing targets", () => {
    const names = listedApps(status, draftFromStatus(status), labels).map(
      (item) => item.packageName,
    )
    expect(names).not.toContain("com.android.settings")
    expect(names).toEqual(
      expect.arrayContaining(["com.google.android.gms", "com.android.vending", "com.gone.app"]),
    )
  })

  it("puts saved targets first, then sorts by label", () => {
    const items = listedApps(status, draftFromStatus(status), labels)
    const saved = items.filter((item) => item.saved).map((item) => item.label)
    expect(items.slice(0, saved.length).every((item) => item.saved)).toBe(true)
    expect(saved).toEqual(["Beta", "com.android.vending", "com.gone.app"])
    const rest = items.filter((item) => !item.saved).map((item) => item.label)
    expect(rest).toEqual(["Alpha", "com.google.android.gms", "Zeta"])
  })

  it("does not reorder rows when the selection changes", () => {
    const draft = draftFromStatus(status)
    const before = listedApps(status, draft, labels).map((item) => item.packageName)
    select(draft, ["com.a.user"])
    expect(listedApps(status, draft, labels).map((item) => item.packageName)).toEqual(before)
  })
})

describe("helpers", () => {
  it("matches labels and package names case-insensitively", () => {
    expect(matches({ label: "Google Wallet", packageName: "com.google.wallet" }, " WALLET ")).toBe(
      true,
    )
    expect(matches({ label: "Google Wallet", packageName: "com.google.wallet" }, "paypal")).toBe(
      false,
    )
  })

  it("counts only real changes", () => {
    const draft = draftFromStatus(status)
    expect(select(draft, ["com.b.target", "com.a.user"])).toBe(1)
    expect(deselect(draft, ["com.a.user", "com.none"])).toBe(1)
  })

  it("formats today for each date placeholder", () => {
    const now = new Date(2026, 8, 5)
    expect(todayValue({ placeholder: "YYYYMM" }, now)).toBe("202609")
    expect(todayValue({ placeholder: "YYYYMMDD" }, now)).toBe("20260905")
    expect(todayValue({ placeholder: "YYYY-MM-DD" }, now)).toBe("2026-09-05")
    expect(todayValue({ placeholder: "today" }, now)).toBeNull()
  })
})
