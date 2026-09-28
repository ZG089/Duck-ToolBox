<script setup lang="ts">
import { computed, ref, watch } from "vue"
import { useI18n } from "vue-i18n"

import { Button } from "@/components/ui/button"
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle,
} from "@/components/ui/drawer"
import { Field, FieldContent, FieldLabel } from "@/components/ui/field"
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group"
import { Switch } from "@/components/ui/switch"
import { useRouteModal } from "@/core/modal"

import type { Policy, TargetMode } from "../api"
import { modes } from "../api"
import { useTrickyI18n } from "../i18n"
import { useTrickyStore } from "../store"
import PolicyEditor from "./PolicyEditor.vue"

/** Tricky Addon's mode dialog: `pkg`, `pkg!` or `pkg?`, plus an optional per-app policy. */
const props = defineProps<{ labels: ReadonlyMap<string, string> }>()
const { open, close, param } = useRouteModal("mode")
const store = useTrickyStore()
const { t } = useTrickyI18n()
const global = useI18n()

const packageName = computed(() => param("pkg") ?? "")
const mode = ref<TargetMode>("auto")
const custom = ref(false)
const policy = ref<Policy>({})

watch(
  () => open.value && packageName.value,
  (name) => {
    if (!name || !store.draft) return
    mode.value = store.draft.targets.get(name) ?? "auto"
    const existing = store.draft.perApp.get(name)
    custom.value = !!existing
    policy.value = { ...(existing ?? store.draft.defaultPolicy) }
  },
  { immediate: true },
)

const modeLabels: Record<TargetMode, () => string> = {
  auto: () => t("ta.mode_auto"),
  generate: () => t("ta.mode_certificate_generating"),
  hack: () => t("ta.mode_leaf_hack"),
}

function save() {
  const hasPolicy = custom.value && Object.keys(policy.value).length > 0
  store.setMode(packageName.value, mode.value, hasPolicy ? policy.value : null)
  close()
}
</script>

<template>
  <Drawer v-model:open="open">
    <DrawerContent class="max-h-[90vh]">
      <div class="mx-auto flex min-h-0 w-full max-w-lg flex-col">
        <DrawerHeader>
          <DrawerTitle>{{ t("ta.mode_dialog_title") }}</DrawerTitle>
          <DrawerDescription class="truncate">
            {{ props.labels.get(packageName) ?? packageName }} ·
            <span class="font-mono">{{ packageName }}</span>
          </DrawerDescription>
        </DrawerHeader>

        <div class="flex min-h-0 flex-col gap-5 overflow-y-auto px-4 pb-2">
          <RadioGroup v-if="store.schema?.supports_app_mode" v-model="mode" class="gap-2">
            <Field
              v-for="option in modes"
              :key="option"
              orientation="horizontal"
              class="rounded-lg border p-3"
            >
              <RadioGroupItem :id="`mode-${option}`" :value="option" />
              <FieldLabel :for="`mode-${option}`" class="flex-1 font-normal">
                {{ modeLabels[option]() }}
                <span v-if="option !== 'auto'" class="text-muted-foreground font-mono">
                  {{ option === "generate" ? "!" : "?" }}
                </span>
              </FieldLabel>
            </Field>
          </RadioGroup>

          <template v-if="store.schema?.supports_per_app_policy">
            <Field orientation="horizontal">
              <FieldContent>
                <FieldLabel for="mode-custom-policy">{{ t("policy.custom") }}</FieldLabel>
              </FieldContent>
              <Switch id="mode-custom-policy" v-model="custom" />
            </Field>
            <PolicyEditor
              v-if="custom"
              v-model="policy"
              :fields="store.schema.default_policy"
              id-prefix="app-policy"
            />
          </template>
        </div>

        <DrawerFooter class="flex-row pb-safe">
          <Button variant="outline" class="flex-1" @click="close()">
            {{ global.t("core.actions.cancel") }}
          </Button>
          <Button class="flex-1" @click="save">{{ t("ta.functional_button_save") }}</Button>
        </DrawerFooter>
      </div>
    </DrawerContent>
  </Drawer>
</template>
