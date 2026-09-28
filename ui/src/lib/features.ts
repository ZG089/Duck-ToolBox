import type { Component } from "vue"

/** A tool shown in the launcher. New features register here and nowhere else. */
export interface FeatureDescriptor {
  id: string
  /** i18n keys resolved by the shell. */
  nameKey: string
  categoryKey: string
  summaryKey: string
  capabilityKeys: string[]
  /** Lazily loaded workbench component, so a feature's code only loads when opened. */
  component: () => Promise<Component>
  /** Shown as a one-time warning dialog before the workbench opens. */
  warningKey?: string
}

export const FEATURES: FeatureDescriptor[] = [
  {
    id: "rkp",
    nameKey: "tool.rkpName",
    categoryKey: "tool.rkpCategory",
    summaryKey: "tool.rkpSummary",
    capabilityKeys: ["workspace.profile", "workspace.provision", "workspace.verify"],
    component: () => import("@/features/rkp/RkpWorkbench.vue"),
  },
  {
    id: "tricky-store",
    nameKey: "tool.trickyStoreName",
    categoryKey: "tool.trickyStoreCategory",
    summaryKey: "tool.trickyStoreSummary",
    capabilityKeys: [
      "trickyStore.capabilityDetect",
      "trickyStore.capabilityTargets",
      "trickyStore.capabilityKeybox",
    ],
    component: () => import("@/features/tricky-store/TrickyStoreWorkbench.vue"),
  },
  {
    id: "device-ids",
    nameKey: "tool.deviceIdsName",
    categoryKey: "tool.deviceIdsCategory",
    summaryKey: "tool.deviceIdsSummary",
    capabilityKeys: [
      "deviceIds.capabilityAutofill",
      "deviceIds.capabilityProvision",
      "deviceIds.capabilityReport",
    ],
    component: () => import("@/features/device-ids/DeviceIdsWorkbench.vue"),
    warningKey: "deviceIds.warningBody",
  },
]
