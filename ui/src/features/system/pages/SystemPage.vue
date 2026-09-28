<script setup lang="ts">
import { computed } from "vue"
import { useRouter } from "vue-router"

import { Button } from "@/components/ui/button"
import KeyValueList from "@/core/components/KeyValueList.vue"
import QueryState from "@/core/components/QueryState.vue"
import SectionCard from "@/core/components/SectionCard.vue"

import ActivityList from "../components/ActivityList.vue"
import ArtifactsCard from "../components/ArtifactsCard.vue"
import MaintenanceCard from "../components/MaintenanceCard.vue"
import UpdateCard from "../components/UpdateCard.vue"
import { useSystemI18n } from "../i18n"
import { useSystemInfo } from "../queries"

const { t, dynamic } = useSystemI18n()
const router = useRouter()
const info = useSystemInfo()

const rows = computed(() => {
  const data = info.data.value
  if (!data) return []
  const manager = data.root_manager
  return [
    { label: t("module.version"), value: `${data.module.version} (${data.module.version_code})` },
    { label: t("module.binary"), value: data.module.binary_version, mono: true },
    { label: t("module.id"), value: data.module.id, mono: true },
    {
      label: t("device.rootManager"),
      value: manager
        ? [dynamic(`rootManagers.${manager.kind}`, manager.kind), manager.version]
            .filter(Boolean)
            .join(" ")
        : t("device.notDetected"),
    },
    { label: t("device.kernel"), value: data.device.kernel_release, mono: true },
    { label: "ABI", value: data.device.abi, mono: true },
  ]
})
</script>

<template>
  <div class="flex flex-col gap-4 p-4">
    <SectionCard :title="t('module.title')">
      <QueryState
        :loading="info.isPending.value"
        :error="info.error.value"
        :skeleton-rows="2"
        @retry="info.refetch()"
      >
        <KeyValueList :rows="rows" />
      </QueryState>
    </SectionCard>

    <UpdateCard />

    <SectionCard :title="t('activity.title')">
      <template #action>
        <Button variant="ghost" size="sm" @click="router.push('/system/activity')">
          {{ t("activity.all") }}
        </Button>
      </template>
      <ActivityList :limit="5" />
    </SectionCard>

    <ArtifactsCard />
    <MaintenanceCard />
  </div>
</template>
