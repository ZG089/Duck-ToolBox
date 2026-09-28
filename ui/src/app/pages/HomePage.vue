<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query"
import { useSessionStorage } from "@vueuse/core"
import { MonitorSmartphone } from "@lucide/vue"
import { computed, ref } from "vue"
import { useI18n } from "vue-i18n"
import { useRouter } from "vue-router"

import { Alert, AlertDescription } from "@/components/ui/alert"
import { ItemGroup } from "@/components/ui/item"
import ConfirmDialog from "@/core/components/ConfirmDialog.vue"
import { bridge } from "@/core/bridge"
import { featureManifest } from "@/core/duckd"
import type { FeatureDefinition } from "@/core/features"
import { availability, contributions, features } from "@/core/features"

import FeatureItem from "../components/FeatureItem.vue"

const { t } = useI18n()
const router = useRouter()
const hostAvailable = bridge().available

const manifest = useQuery({
  queryKey: ["features"],
  queryFn: featureManifest,
  enabled: hostAvailable,
  staleTime: Number.POSITIVE_INFINITY,
})

const widgets = contributions("homeWidgets")
const groups = computed(() =>
  (["tool", "system"] as const)
    .map((placement) => ({
      placement,
      items: features().filter((feature) => (feature.placement ?? "tool") === placement),
    }))
    .filter((group) => group.items.length > 0),
)

function state(feature: FeatureDefinition) {
  return hostAvailable ? availability(feature, manifest.data.value) : "available"
}

const acknowledged = useSessionStorage<string[]>("duck-toolbox/acknowledged", [])
const pending = ref<FeatureDefinition | null>(null)
const confirmOpen = computed({
  get: () => pending.value !== null,
  set: (value) => {
    if (!value) pending.value = null
  },
})

function open(feature: FeatureDefinition) {
  if (feature.confirmOpen && !acknowledged.value.includes(feature.id)) {
    pending.value = feature
    return
  }
  void router.push(`/${feature.id}`)
}

function confirm() {
  const feature = pending.value
  if (!feature) return
  acknowledged.value = [...acknowledged.value, feature.id]
  pending.value = null
  void router.push(`/${feature.id}`)
}
</script>

<template>
  <div class="flex flex-col gap-5 p-4">
    <Alert v-if="!hostAvailable">
      <MonitorSmartphone />
      <AlertDescription>{{ t("core.home.noHost") }}</AlertDescription>
    </Alert>

    <component :is="widget" v-for="(widget, index) in widgets" :key="index" />

    <section v-for="group in groups" :key="group.placement" class="flex flex-col gap-2">
      <h2 class="text-muted-foreground px-1 text-sm font-medium">
        {{ group.placement === "tool" ? t("core.home.tools") : t("core.home.system") }}
      </h2>
      <ItemGroup class="gap-2">
        <FeatureItem
          v-for="feature in group.items"
          :key="feature.id"
          :feature="feature"
          :availability="state(feature)"
          @open="open"
        />
      </ItemGroup>
    </section>

    <ConfirmDialog
      v-model:open="confirmOpen"
      :title="t('core.home.confirmTitle')"
      :description="pending ? t(`${pending.namespace}.meta.warning`) : ''"
      :confirm-label="t('core.actions.continue')"
      @confirm="confirm"
    />
  </div>
</template>
