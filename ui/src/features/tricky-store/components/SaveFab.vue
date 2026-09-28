<script setup lang="ts">
import { NotebookPen, Undo2 } from "@lucide/vue"
import { useWindowScroll } from "@vueuse/core"
import { ref, watch } from "vue"

import { Button } from "@/components/ui/button"
import { Spinner } from "@/components/ui/spinner"

import { useTrickyI18n } from "../i18n"

defineProps<{ dirty: boolean; saving: boolean }>()
const emit = defineEmits<{ save: []; discard: [] }>()
const { t } = useTrickyI18n()

// Like Tricky Addon, the button gets out of the way while scrolling down the list.
const { y } = useWindowScroll()
const hidden = ref(false)
watch(y, (now, before) => (hidden.value = now > before && now > 48))
</script>

<template>
  <div
    :class="[
      'bottom-safe pointer-events-none fixed inset-x-0 z-30 mb-4 flex justify-end px-4 transition-transform duration-200',
      hidden ? 'translate-y-[calc(100%+2rem)]' : '',
    ]"
  >
    <div class="pointer-events-auto mx-auto flex w-full max-w-3xl items-center justify-end gap-2">
      <Button v-if="dirty" variant="secondary" class="shadow-md" @click="emit('discard')">
        <Undo2 />
        {{ t("list.discard") }}
      </Button>
      <Button size="lg" class="h-12 rounded-2xl shadow-lg" :disabled="saving" @click="emit('save')">
        <Spinner v-if="saving" />
        <NotebookPen v-else />
        {{ t("ta.functional_button_save") }}
        <span
          v-if="dirty"
          class="bg-primary-foreground size-2 rounded-full"
          :aria-label="t('list.unsaved')"
        />
      </Button>
    </div>
  </div>
</template>
