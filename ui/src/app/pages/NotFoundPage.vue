<script setup lang="ts">
import { useI18n } from "vue-i18n"

import { Button } from "@/components/ui/button"

const { t } = useI18n()
</script>

<template>
  <div class="flex flex-col items-center gap-4 p-10 text-center">
    <p class="text-muted-foreground">{{ t("core.nav.notFound") }}</p>
    <Button as-child
      ><RouterLink to="/">{{ t("core.nav.home") }}</RouterLink></Button
    >
  </div>
</template>
