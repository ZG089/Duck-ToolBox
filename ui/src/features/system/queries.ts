import { useQuery } from "@tanstack/vue-query"
import { useStorage } from "@vueuse/core"
import { computed } from "vue"

import { bridge } from "@/core/bridge"

import type { UpdateChannel } from "./api"
import { systemApi } from "./api"

export function useSystemInfo() {
  return useQuery({
    queryKey: ["system.info"],
    queryFn: systemApi.info,
    enabled: bridge().available,
    staleTime: 5 * 60_000,
  })
}

/** `off` skips the automatic check on the home page, like Tricky Addon's "Disable". */
export const updateChannel = useStorage<UpdateChannel | "off">(
  "duck-toolbox/update-channel",
  "stable",
)

export function useUpdateCheck(enabled = computed(() => true)) {
  return useQuery({
    queryKey: ["system.update", updateChannel],
    queryFn: () => systemApi.checkUpdate(updateChannel.value === "canary" ? "canary" : "stable"),
    enabled: computed(() => bridge().available && updateChannel.value !== "off" && enabled.value),
    staleTime: 30 * 60_000,
  })
}
