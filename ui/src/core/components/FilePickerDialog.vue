<script setup lang="ts">
import { useQuery } from "@tanstack/vue-query"
import { ArrowUp, File, Folder, FolderOpen } from "@lucide/vue"
import { computed, ref, useTemplateRef, watch } from "vue"
import { useI18n } from "vue-i18n"

import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Item, ItemContent, ItemDescription, ItemMedia, ItemTitle } from "@/components/ui/item"
import { listFiles } from "@/core/duckd"
import { formatBytes, formatDateTime } from "@/core/format"

import QueryState from "./QueryState.vue"

/** A chosen file: a device path from the browser, or content from the system picker. */
export type PickedFile = { path: string } | { name: string; content: string }

const props = withDefaults(
  defineProps<{ extension: string; title?: string; start?: string; root?: string }>(),
  {
    title: undefined,
    start: "/storage/emulated/0/Download",
    // Like Tricky Addon's picker, browsing stops at shared storage by default.
    root: "/storage/emulated/0",
  },
)
const open = defineModel<boolean>("open", { required: true })
const emit = defineEmits<{ pick: [file: PickedFile] }>()

const { t } = useI18n()
const directory = ref(props.start)
const input = useTemplateRef<HTMLInputElement>("input")

watch(open, (isOpen) => {
  if (isOpen) directory.value = props.start
})

const listing = useQuery({
  queryKey: ["system.files", directory, () => props.extension],
  queryFn: () => listFiles(directory.value, props.extension),
  enabled: open,
  staleTime: 0,
})

const segments = computed(() =>
  directory.value
    .slice(props.root.length)
    .split("/")
    .filter(Boolean)
    .map((name, index, all) => ({
      name,
      path: `${props.root}/${all.slice(0, index + 1).join("/")}`,
    })),
)

function up() {
  if (directory.value === props.root) return
  directory.value = directory.value.split("/").slice(0, -1).join("/") || props.root
}

function choose(path: string, isDirectory: boolean) {
  if (isDirectory) {
    directory.value = path
    return
  }
  emit("pick", { path })
  open.value = false
}

async function onSystemFile(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return
  emit("pick", { name: file.name, content: await file.text() })
  open.value = false
}
</script>

<template>
  <Dialog v-model:open="open">
    <DialogContent class="flex max-h-[85vh] flex-col gap-3">
      <DialogHeader>
        <DialogTitle>{{ title ?? t("core.files.title") }}</DialogTitle>
        <DialogDescription class="flex flex-wrap items-center gap-1 font-mono text-xs">
          <button type="button" class="hover:underline" @click="directory = root">
            {{ root }}
          </button>
          <template v-for="segment in segments" :key="segment.path">
            <span aria-hidden="true">/</span>
            <button type="button" class="hover:underline" @click="directory = segment.path">
              {{ segment.name }}
            </button>
          </template>
        </DialogDescription>
      </DialogHeader>

      <div class="-mx-2 min-h-40 flex-1 overflow-y-auto px-2">
        <QueryState
          :loading="listing.isPending.value"
          :error="listing.error.value"
          @retry="listing.refetch()"
        >
          <div class="flex flex-col gap-1">
            <Item
              v-if="directory !== root"
              as="button"
              size="sm"
              class="w-full text-start"
              @click="up"
            >
              <ItemMedia variant="icon"><ArrowUp /></ItemMedia>
              <ItemContent
                ><ItemTitle>{{ t("core.files.up") }}</ItemTitle></ItemContent
              >
            </Item>
            <Item
              v-for="entry in listing.data.value?.entries ?? []"
              :key="entry.path"
              as="button"
              size="sm"
              class="w-full text-start"
              @click="choose(entry.path, entry.directory)"
            >
              <ItemMedia variant="icon">
                <Folder v-if="entry.directory" />
                <File v-else />
              </ItemMedia>
              <ItemContent class="min-w-0">
                <ItemTitle class="w-full truncate">{{ entry.name }}</ItemTitle>
                <ItemDescription v-if="!entry.directory">
                  {{ formatBytes(entry.size) }} · {{ formatDateTime(entry.modified_unix) }}
                </ItemDescription>
              </ItemContent>
            </Item>
            <p
              v-if="listing.data.value && listing.data.value.entries.length === 0"
              class="text-muted-foreground py-6 text-center text-sm"
            >
              {{ t("core.files.empty") }}
            </p>
          </div>
        </QueryState>
      </div>

      <DialogFooter class="flex-row justify-between gap-2 sm:justify-between">
        <Button variant="outline" @click="input?.click()">
          <FolderOpen />
          {{ t("core.files.system") }}
        </Button>
        <Button variant="ghost" @click="open = false">{{ t("core.actions.cancel") }}</Button>
      </DialogFooter>
      <input
        ref="input"
        type="file"
        class="hidden"
        :accept="extension ? `.${extension}` : undefined"
        @change="onSystemFile"
      />
    </DialogContent>
  </Dialog>
</template>
