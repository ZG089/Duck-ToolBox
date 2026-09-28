<script setup lang="ts">
import { ref, watch } from "vue"

import { useI18n } from "@/i18n"
import type { TrickyStoreController } from "../useTrickyStore"

const props = defineProps<{ controller: TrickyStoreController; open: boolean }>()
const emit = defineEmits<{ close: [] }>()
const { t } = useI18n()
const { state, actions } = props.controller

const handlerEnabled = ref(true)
const bootHash = ref("")

watch(
  () => props.open,
  (open) => {
    if (open) {
      handlerEnabled.value = state.status?.props.prop_handler_enabled ?? true
      bootHash.value = state.status?.props.boot_hash ?? ""
    }
  },
)

async function save() {
  await actions.saveProps(handlerEnabled.value, bootHash.value.trim() || null)
  emit("close")
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="dialog-backdrop" @click.self="emit('close')">
      <section class="dialog-card" role="dialog" aria-modal="true">
        <div class="panel-heading compact">
          <h3 class="panel-title dialog-title">{{ t("trickyStore.propsTitle") }}</h3>
        </div>

        <label class="switch-row">
          <input v-model="handlerEnabled" type="checkbox">
          <span>
            <strong>{{ t("trickyStore.propHandler") }}</strong>
            <small>{{ t("trickyStore.propHandlerDescription") }}</small>
          </span>
        </label>

        <div class="field-group mt-4">
          <label class="field-label" for="boot-hash">{{ t("trickyStore.bootHash") }}</label>
          <textarea
            id="boot-hash"
            v-model="bootHash"
            class="text-input"
            rows="3"
            spellcheck="false"
            placeholder="241890bd44131d34c077cb01a0c3ea1ff68533b21e9d83b3f3adca6663c3d443"
          />
          <small class="field-hint">{{ t("trickyStore.bootHashDescription") }}</small>
        </div>

        <div class="toolbar mt-4">
          <button class="action-primary" type="button" :disabled="state.busy.props" @click="save">
            {{ t("functional.save") }}
          </button>
          <button class="action-secondary" type="button" @click="emit('close')">{{ t("dialog.close") }}</button>
        </div>
      </section>
    </div>
  </Teleport>
</template>
