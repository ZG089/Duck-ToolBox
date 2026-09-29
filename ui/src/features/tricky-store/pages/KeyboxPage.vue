<script setup lang="ts">
import { computed } from "vue"

import KeyValueList from "@/core/components/KeyValueList.vue"
import QueryState from "@/core/components/QueryState.vue"
import SectionCard from "@/core/components/SectionCard.vue"
import { formatBytes, formatDateTime } from "@/core/format"

import KeyboxSources from "../components/KeyboxSources.vue"
import ProvidersCard from "../components/ProvidersCard.vue"
import { useTrickyI18n } from "../i18n"
import { useStatusQuery } from "../queries"

const { t } = useTrickyI18n()
const status = useStatusQuery()

const rows = computed(() => {
  const keybox = status.data.value?.keybox
  if (!keybox?.exists) return []
  return [
    { label: t("keybox.path"), value: keybox.path, mono: true, copy: true },
    { label: t("keybox.size"), value: formatBytes(keybox.size) },
    { label: t("keybox.modified"), value: formatDateTime(keybox.modified_unix) },
  ]
})
</script>

<template>
  <div class="flex flex-col gap-6 p-4">
    <QueryState
      :loading="status.isPending.value"
      :error="status.error.value"
      @retry="status.refetch()"
    >
      <SectionCard
        :title="status.data.value?.keybox?.exists ? t('keybox.status') : t('keybox.missing')"
      >
        <KeyValueList v-if="rows.length" :rows="rows" />
        <p v-else class="text-muted-foreground text-sm">{{ status.data.value?.keybox?.path }}</p>
      </SectionCard>
      <KeyboxSources />
      <ProvidersCard />
    </QueryState>
  </div>
</template>
