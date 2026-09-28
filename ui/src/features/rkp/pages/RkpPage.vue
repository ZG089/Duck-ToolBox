<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query"
import { useRouteQuery } from "@vueuse/router"

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import QueryState from "@/core/components/QueryState.vue"

import KeyboxTab from "../components/KeyboxTab.vue"
import ProfileTab from "../components/ProfileTab.vue"
import ProvisionTab from "../components/ProvisionTab.vue"
import VerifyTab from "../components/VerifyTab.vue"
import { useRkpI18n } from "../i18n"
import { useRkpStore } from "../store"

const { t } = useRkpI18n()
const store = useRkpStore()
const tab = useRouteQuery("tab", "profile", { mode: "replace" })

const loading = useQuery({
  queryKey: ["rkp.profile"],
  queryFn: async () => {
    await store.load()
    return true
  },
  staleTime: Number.POSITIVE_INFINITY,
})

const tabs = ["profile", "provision", "keybox", "verify"] as const
</script>

<template>
  <div class="flex flex-col gap-4 p-4">
    <QueryState
      :loading="loading.isPending.value"
      :error="loading.error.value"
      @retry="loading.refetch()"
    >
      <Tabs v-model="tab" class="gap-4">
        <TabsList class="w-full">
          <TabsTrigger v-for="name in tabs" :key="name" :value="name">
            {{ t(`tabs.${name}`) }}
          </TabsTrigger>
        </TabsList>
        <TabsContent value="profile"><ProfileTab /></TabsContent>
        <TabsContent value="provision"><ProvisionTab /></TabsContent>
        <TabsContent value="keybox"><KeyboxTab /></TabsContent>
        <TabsContent value="verify"><VerifyTab /></TabsContent>
      </Tabs>
    </QueryState>
  </div>
</template>
