import { useQuery } from "@tanstack/vue-query"
import type { Ref } from "vue"
import { computed } from "vue"

import { bridge } from "@/core/bridge"

import { trickyStoreApi } from "./api"

export const STATUS_KEY = ["tricky-store", "status"] as const

export function useStatusQuery() {
  return useQuery({ queryKey: STATUS_KEY, queryFn: trickyStoreApi.status })
}

/** App labels from the root manager; apps it does not know keep their package name. */
export function useLabels(packages: Ref<readonly string[]>) {
  const query = useQuery({
    queryKey: computed(() => ["tricky-store", "labels", packages.value] as const),
    queryFn: async () => {
      const info = await bridge().packagesInfo([...packages.value])
      return new Map(info.map((entry) => [entry.packageName, entry.label]))
    },
    enabled: computed(() => packages.value.length > 0),
    staleTime: Number.POSITIVE_INFINITY,
    placeholderData: (previous) => previous,
  })
  return computed<ReadonlyMap<string, string>>(() => query.data.value ?? new Map())
}

export function useProvidersQuery() {
  return useQuery({ queryKey: ["tricky-store", "providers"], queryFn: trickyStoreApi.providers })
}

export function useEntryQuery() {
  return useQuery({ queryKey: ["tricky-store", "entry"], queryFn: trickyStoreApi.entry })
}
