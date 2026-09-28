<script setup lang="ts">
import { CloudDownload } from "@lucide/vue"
import { useRouter } from "vue-router"

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Button } from "@/components/ui/button"

import { useSystemI18n } from "../i18n"
import { useUpdateCheck } from "../queries"

const { t } = useSystemI18n()
const router = useRouter()
const update = useUpdateCheck()
</script>

<template>
  <Alert v-if="update.data.value?.available" class="items-center">
    <CloudDownload />
    <AlertTitle>{{ t("update.banner", { version: update.data.value.version ?? "" }) }}</AlertTitle>
    <AlertDescription class="flex justify-end">
      <Button size="sm" @click="router.push({ path: '/system', hash: '#update' })">
        {{ t("update.view") }}
      </Button>
    </AlertDescription>
  </Alert>
</template>
