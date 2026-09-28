<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query"
import { Save } from "@lucide/vue"
import { ref, watch } from "vue"

import { Button } from "@/components/ui/button"
import { Spinner } from "@/components/ui/spinner"
import QueryState from "@/core/components/QueryState.vue"
import SectionCard from "@/core/components/SectionCard.vue"
import { notifySuccess } from "@/core/notify"

import type { Policy } from "../api"
import PolicyEditor from "../components/PolicyEditor.vue"
import { useTrickyI18n } from "../i18n"
import { useStatusQuery } from "../queries"
import { useTrickyStore } from "../store"

const { t } = useTrickyI18n()
const store = useTrickyStore()
const status = useStatusQuery()
const policy = ref<Policy>({})

watch(
  status.data,
  (next) => {
    if (!next) return
    store.load(next)
    policy.value = { ...next.config.default_policy }
  },
  { immediate: true },
)

const save = useMutation({
  mutationFn: async () => {
    const saved = await store.saveOnly({ defaultPolicy: policy.value })
    const next = await status.refetch()
    if (next.data) store.load(next.data)
    return saved
  },
  onSuccess: (saved) =>
    notifySuccess(t("policy.saved"), saved.restart_required ? t("policy.restart") : undefined),
})
</script>

<template>
  <div class="flex flex-col gap-4 p-4">
    <QueryState
      :loading="status.isPending.value"
      :error="status.error.value"
      @retry="status.refetch()"
    >
      <p v-if="!store.schema?.default_policy.length" class="text-muted-foreground text-sm">
        {{ t("policy.none") }}
      </p>
      <template v-else>
        <SectionCard :description="t('policy.hint')">
          <PolicyEditor
            v-model="policy"
            :fields="store.schema.default_policy"
            id-prefix="default-policy"
          />
        </SectionCard>
        <Button :disabled="save.isPending.value" @click="save.mutate()">
          <Spinner v-if="save.isPending.value" />
          <Save v-else />
          {{ t("ta.functional_button_save") }}
        </Button>
      </template>
    </QueryState>
  </div>
</template>
