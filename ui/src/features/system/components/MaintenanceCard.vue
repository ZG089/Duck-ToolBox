<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query"
import { Power, Trash2 } from "@lucide/vue"
import { ref } from "vue"

import { Item, ItemContent, ItemGroup, ItemMedia, ItemTitle } from "@/components/ui/item"
import ConfirmDialog from "@/core/components/ConfirmDialog.vue"
import SectionCard from "@/core/components/SectionCard.vue"
import { notifySuccess } from "@/core/notify"

import { systemApi } from "../api"
import { useSystemI18n } from "../i18n"

const { t } = useSystemI18n()
const confirmReboot = ref(false)
const confirmUninstall = ref(false)

const reboot = useMutation({ mutationFn: systemApi.reboot })
const uninstall = useMutation({
  mutationFn: systemApi.uninstall,
  onSuccess: () => notifySuccess(t("maintenance.uninstalled")),
})
</script>

<template>
  <SectionCard :title="t('maintenance.title')" plain>
    <ItemGroup>
      <Item
        as="button"
        variant="segmented"
        size="sm"
        class="state-layer relative w-full text-start"
        @click="confirmReboot = true"
      >
        <ItemMedia variant="icon"><Power /></ItemMedia>
        <ItemContent>
          <ItemTitle>{{ t("maintenance.reboot") }}</ItemTitle>
        </ItemContent>
      </Item>
      <Item
        as="button"
        variant="segmented"
        size="sm"
        class="state-layer text-destructive relative w-full text-start disabled:opacity-50"
        :disabled="uninstall.isSuccess.value"
        @click="confirmUninstall = true"
      >
        <ItemMedia variant="icon" class="bg-destructive/10 text-destructive"><Trash2 /></ItemMedia>
        <ItemContent>
          <ItemTitle>{{ t("maintenance.uninstall") }}</ItemTitle>
        </ItemContent>
      </Item>
    </ItemGroup>
    <ConfirmDialog
      v-model:open="confirmReboot"
      :title="t('maintenance.rebootConfirm')"
      :confirm-label="t('maintenance.reboot')"
      @confirm="reboot.mutate()"
    />
    <ConfirmDialog
      v-model:open="confirmUninstall"
      :title="t('maintenance.uninstall')"
      :description="t('maintenance.uninstallConfirm')"
      :confirm-label="t('maintenance.uninstall')"
      destructive
      @confirm="uninstall.mutate()"
    />
  </SectionCard>
</template>
