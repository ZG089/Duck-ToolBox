<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query"
import { KeyRound, Save, Trash2 } from "@lucide/vue"
import { storeToRefs } from "pinia"
import { computed, ref } from "vue"

import { Button } from "@/components/ui/button"
import { Spinner } from "@/components/ui/spinner"
import ConfirmDialog from "@/core/components/ConfirmDialog.vue"
import KeyValueList from "@/core/components/KeyValueList.vue"
import SectionCard from "@/core/components/SectionCard.vue"
import { notifySuccess } from "@/core/notify"

import { useRkpI18n } from "../i18n"
import { useRkpStore } from "../store"
import AdvancedCard from "./AdvancedCard.vue"
import DeviceCard from "./DeviceCard.vue"
import KeySourceCard from "./KeySourceCard.vue"

const { t } = useRkpI18n()
const store = useRkpStore()
const { draft, info, secretsPath } = storeToRefs(store)
const confirmClear = ref(false)

const save = useMutation({
  mutationFn: store.save,
  onSuccess: () => notifySuccess(t("profile.saved")),
})
const check = useMutation({ mutationFn: store.checkKey })
const clear = useMutation({
  mutationFn: store.clear,
  onSuccess: () => notifySuccess(t("profile.cleared")),
})

const infoRows = computed(() =>
  info.value
    ? [
        { label: t("profile.mode"), value: info.value.mode },
        { label: t("profile.curve"), value: info.value.curve },
        { label: t("profile.publicKey"), value: info.value.public_key_hex, mono: true, copy: true },
        { label: t("profile.seedHex"), value: info.value.seed_hex, mono: true, copy: true },
      ]
    : [],
)
</script>

<template>
  <div v-if="draft" class="flex flex-col gap-6">
    <KeySourceCard v-model="draft" />
    <DeviceCard v-model="draft" />
    <AdvancedCard v-model="draft" />

    <SectionCard v-if="info" :title="t('profile.check')">
      <KeyValueList :rows="infoRows" />
    </SectionCard>

    <p class="text-muted-foreground px-4 text-xs break-all">
      {{ t("profile.secretsNote", { path: secretsPath }) }}
    </p>

    <div class="flex flex-wrap gap-3">
      <Button size="lg" class="flex-1" :disabled="save.isPending.value" @click="save.mutate()">
        <Spinner v-if="save.isPending.value" class="size-6" />
        <Save v-else />
        {{ t("profile.save") }}
      </Button>
      <Button
        variant="secondary"
        size="lg"
        class="flex-1"
        :disabled="check.isPending.value"
        @click="check.mutate()"
      >
        <Spinner v-if="check.isPending.value" class="size-6" />
        <KeyRound v-else />
        {{ t("profile.check") }}
      </Button>
      <Button variant="ghost" class="text-destructive w-full" @click="confirmClear = true">
        <Trash2 />
        {{ t("profile.clear") }}
      </Button>
    </div>

    <ConfirmDialog
      v-model:open="confirmClear"
      :title="t('profile.clearConfirm')"
      :confirm-label="t('profile.clear')"
      destructive
      @confirm="clear.mutate()"
    />
  </div>
</template>
