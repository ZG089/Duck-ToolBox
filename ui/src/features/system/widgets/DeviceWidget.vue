<script setup lang="ts">
import { Cpu, ShieldCheck, Smartphone } from "@lucide/vue"
import { computed } from "vue"

import { Badge } from "@/components/ui/badge"
import { Card, CardContent } from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton"

import { useSystemI18n } from "../i18n"
import { useSystemInfo } from "../queries"

const { t, dynamic } = useSystemI18n()
const info = useSystemInfo()

const device = computed(() => info.data.value?.device)
const manager = computed(() => {
  const root = info.data.value?.root_manager
  if (!root) return t("device.notDetected")
  const name = dynamic(`rootManagers.${root.kind}`, root.kind)
  return root.version ? `${name} ${root.version}` : name
})
</script>

<template>
  <Card v-if="info.isEnabled.value" class="gap-3 py-4">
    <CardContent class="flex flex-col gap-3 px-4">
      <template v-if="device">
        <div class="flex items-center gap-3">
          <div
            class="bg-secondary text-secondary-foreground grid size-10 place-items-center rounded-xl"
          >
            <Smartphone class="size-5" />
          </div>
          <div class="min-w-0 flex-1">
            <p class="truncate font-semibold">{{ device.brand }} {{ device.model }}</p>
            <p class="text-muted-foreground truncate text-sm">
              {{ t("device.android", { release: device.android_release, sdk: device.sdk }) }}
            </p>
          </div>
          <Badge variant="secondary" class="shrink-0"
            ><bdi>{{ info.data.value?.module.version }}</bdi></Badge
          >
        </div>
        <div class="flex flex-wrap gap-2">
          <Badge variant="outline"><ShieldCheck /> {{ manager }}</Badge>
          <Badge v-if="device.security_patch" variant="outline">
            {{ t("device.patch") }} <bdi>{{ device.security_patch }}</bdi>
          </Badge>
          <Badge v-if="device.kernel_release" variant="outline" class="max-w-full">
            <Cpu /> <bdi class="truncate">{{ device.kernel_release }}</bdi>
          </Badge>
          <Badge v-if="device.selinux" variant="outline">
            SELinux · {{ dynamic(`selinux.${device.selinux}`, device.selinux) }}
          </Badge>
        </div>
      </template>
      <template v-else>
        <Skeleton class="h-10 w-2/3" />
        <Skeleton class="h-6 w-full" />
      </template>
    </CardContent>
  </Card>
</template>
