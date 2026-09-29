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

// Material 3 extended FABs shrink to their icon while the list scrolls down, so they stay in
// reach without covering it, and grow back on the way up.
const { y } = useWindowScroll()
const collapsed = ref(false)
watch(y, (now, before) => (collapsed.value = now > before && now > 48))

const fab = "rounded-lg gap-0 px-4 shadow-lg"
// Collapsing only narrows the label, so screen readers still announce it.
const label = "overflow-hidden transition-[max-width,margin,opacity] duration-420 ease-spring"
</script>

<template>
  <div class="bottom-safe pointer-events-none fixed inset-x-0 z-30 mb-4 flex justify-end px-4">
    <div class="pointer-events-auto mx-auto flex w-full max-w-3xl items-center justify-end gap-3">
      <Button v-if="dirty" variant="secondary" size="lg" :class="fab" @click="emit('discard')">
        <Undo2 />
        <span :class="[label, collapsed ? 'ms-0 max-w-0 opacity-0' : 'ms-3 max-w-48']">
          {{ t("list.discard") }}
        </span>
      </Button>
      <Button size="lg" :class="fab" :disabled="saving" @click="emit('save')">
        <Spinner v-if="saving" class="size-6" />
        <NotebookPen v-else />
        <span :class="[label, collapsed ? 'ms-0 max-w-0 opacity-0' : 'ms-3 max-w-48']">
          {{ t("ta.functional_button_save") }}
        </span>
        <span
          v-if="dirty"
          class="bg-primary-foreground absolute end-2.5 top-2.5 size-2 rounded-full"
          :aria-label="t('list.unsaved')"
        />
      </Button>
    </div>
  </div>
</template>
