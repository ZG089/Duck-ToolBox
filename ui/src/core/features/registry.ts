import { shallowRef } from "vue"

import type { FeatureManifest } from "@/core/duckd"

import type { Contribution, ContributionKey, FeatureDefinition } from "./types"

const registered = shallowRef<readonly FeatureDefinition[]>([])

/** Called once by the app shell with every feature it discovered. */
export function registerFeatures(features: readonly FeatureDefinition[]): void {
  const ids = new Set<string>()
  for (const feature of features) {
    if (ids.has(feature.id)) throw new Error(`duplicate feature id: ${feature.id}`)
    ids.add(feature.id)
  }
  registered.value = [...features].sort((left, right) => left.order - right.order)
}

export function features(): readonly FeatureDefinition[] {
  return registered.value
}

export function findFeature(id: string): FeatureDefinition | undefined {
  return registered.value.find((feature) => feature.id === id)
}

/** Every value contributed to `key`, in feature order. */
export function contributions<K extends ContributionKey>(key: K): Contribution<K>[] {
  return registered.value.flatMap(
    (feature) => (feature.contributes?.[key] ?? []) as Contribution<K>[],
  )
}

export type Availability = "available" | "missing" | "outdated" | "unknown"

/**
 * Whether the installed backend can serve `feature`. A missing feature or a different
 * contract version disables the tool instead of letting it misread data.
 */
export function availability(
  feature: FeatureDefinition,
  manifest: FeatureManifest | undefined,
): Availability {
  const backend = feature.backend === undefined ? feature.id : feature.backend
  if (backend === null) return "available"
  if (!manifest) return "unknown"
  const entry = manifest.features.find((candidate) => candidate.id === backend)
  if (!entry) return "missing"
  return entry.contract === (feature.contract ?? 1) ? "available" : "outdated"
}
