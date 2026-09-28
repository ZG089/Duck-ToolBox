<script setup lang="ts">
import { ChevronRight, FileKey, Globe, HardDrive, ShieldQuestion } from "@lucide/vue"
import type { Component } from "vue"
import { computed, ref } from "vue"
import { useRouter } from "vue-router"

import {
  Item,
  ItemActions,
  ItemContent,
  ItemDescription,
  ItemGroup,
  ItemMedia,
  ItemTitle,
} from "@/components/ui/item"
import { Spinner } from "@/components/ui/spinner"
import FilePickerDialog from "@/core/components/FilePickerDialog.vue"
import type { PickedFile } from "@/core/components/FilePickerDialog.vue"
import SectionCard from "@/core/components/SectionCard.vue"

import { trickyStoreApi } from "../api"
import { useTrickyI18n } from "../i18n"
import { useKeyboxInstall } from "../keybox"

const { t } = useTrickyI18n()
const router = useRouter()
const picking = ref(false)

const aosp = useKeyboxInstall(trickyStoreApi.keyboxAosp, { success: "ta.prompt_aosp_key_set" })
const unknown = useKeyboxInstall(trickyStoreApi.keyboxGenerate, {
  success: "ta.prompt_unknown_key_set",
})
const local = useKeyboxInstall(
  (file: PickedFile) =>
    "path" in file
      ? trickyStoreApi.keyboxInstall(file.path)
      : trickyStoreApi.keyboxImport(file.content),
  { success: "ta.prompt_custom_key_set" },
)

interface Source {
  id: string
  icon: Component
  title: string
  description: string
  busy: () => boolean
  run: () => void
}

const sources = computed<Source[]>(() => [
  {
    id: "aosp",
    icon: FileKey,
    title: t("ta.menu_keybox_aosp"),
    description: t("keybox.aospHint"),
    busy: () => aosp.isPending.value,
    run: () => aosp.mutate(undefined),
  },
  {
    id: "unknown",
    icon: ShieldQuestion,
    title: t("ta.menu_keybox_unknown"),
    description: t("keybox.unknownHint"),
    busy: () => unknown.isPending.value,
    run: () => unknown.mutate(undefined),
  },
  {
    id: "local",
    icon: HardDrive,
    title: t("ta.menu_keybox_local"),
    description: t("keybox.localHint"),
    busy: () => local.isPending.value,
    run: () => (picking.value = true),
  },
  {
    id: "repo",
    icon: Globe,
    title: t("ta.menu_keybox_repo"),
    description: t("keybox.repoHint"),
    busy: () => false,
    run: () => void router.push("/tricky-store/keybox/repo"),
  },
])
</script>

<template>
  <SectionCard :title="t('keybox.sources')" content-class="px-2">
    <ItemGroup>
      <Item v-for="source in sources" :key="source.id" as-child size="sm">
        <button
          type="button"
          class="w-full text-start"
          :disabled="source.busy()"
          @click="source.run()"
        >
          <ItemMedia variant="icon"><component :is="source.icon" /></ItemMedia>
          <ItemContent>
            <ItemTitle>{{ source.title }}</ItemTitle>
            <ItemDescription>{{ source.description }}</ItemDescription>
          </ItemContent>
          <ItemActions>
            <Spinner v-if="source.busy()" />
            <ChevronRight v-else class="rtl-flip text-muted-foreground size-4" />
          </ItemActions>
        </button>
      </Item>
    </ItemGroup>
    <FilePickerDialog v-model:open="picking" extension="xml" @pick="local.mutate($event)" />
  </SectionCard>
</template>
