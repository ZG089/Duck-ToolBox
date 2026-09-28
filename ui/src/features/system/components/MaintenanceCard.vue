<script setup lang="ts">
import { useMutation } from "@tanstack/vue-query"
import { Power, Trash2 } from "@lucide/vue"
import { ref } from "vue"

import { Button } from "@/components/ui/button"
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
  <SectionCard :title="t('maintenance.title')">
    <div class="flex flex-col gap-2">
      <Button variant="outline" class="justify-start" @click="confirmReboot = true">
        <Power />
        {{ t("maintenance.reboot") }}
      </Button>
      <Button
        variant="outline"
        class="text-destructive justify-start"
        :disabled="uninstall.isSuccess.value"
        @click="confirmUninstall = true"
      >
        <Trash2 />
        {{ t("maintenance.uninstall") }}
      </Button>
    </div>
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
