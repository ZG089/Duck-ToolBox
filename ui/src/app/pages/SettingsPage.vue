<script setup lang="ts">
import { ExternalLink, Monitor, Moon, Sun } from "@lucide/vue"
import { computed } from "vue"
import { useI18n } from "vue-i18n"

import { Button } from "@/components/ui/button"
import { Field, FieldLabel } from "@/components/ui/field"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group"
import SectionCard from "@/core/components/SectionCard.vue"
import { contributions } from "@/core/features"
import { availableLocales, displayName, useLocalePreference } from "@/core/i18n"
import { openExternal, REPOSITORY_URL } from "@/core/links"
import type { ThemePreference } from "@/core/theme"
import { useThemePreference } from "@/core/theme"

import { ACKNOWLEDGEMENTS, AUTHORS } from "../meta"

const { t } = useI18n()
const theme = useThemePreference()
const locale = useLocalePreference()
const sections = contributions("settingsSections")

const themes = computed(() => [
  { value: "system" as const, label: t("core.settings.themeSystem"), icon: Monitor },
  { value: "light" as const, label: t("core.settings.themeLight"), icon: Sun },
  { value: "dark" as const, label: t("core.settings.themeDark"), icon: Moon },
])
const locales = computed(() =>
  availableLocales().map((value) => ({ value, label: displayName(value) })),
)

function setTheme(value: unknown) {
  if (value === "system" || value === "light" || value === "dark") {
    theme.value = value satisfies ThemePreference
  }
}
</script>

<template>
  <div class="flex flex-col gap-6 p-4">
    <SectionCard :title="t('core.settings.appearance')">
      <div class="flex flex-col gap-6">
        <Field>
          <FieldLabel>{{ t("core.settings.theme") }}</FieldLabel>
          <ToggleGroup
            type="single"
            variant="tonal"
            class="w-full"
            :model-value="theme"
            @update:model-value="setTheme"
          >
            <!-- Icon above the label, so the three fit a small phone in any language. -->
            <ToggleGroupItem
              v-for="option in themes"
              :key="option.value"
              :value="option.value"
              class="h-auto min-h-16 flex-1 flex-col gap-1 px-2 py-2.5 text-center leading-tight whitespace-normal"
            >
              <component :is="option.icon" />
              {{ option.label }}
            </ToggleGroupItem>
          </ToggleGroup>
        </Field>
        <Field>
          <FieldLabel for="language">{{ t("core.settings.language") }}</FieldLabel>
          <Select v-model="locale">
            <SelectTrigger id="language" class="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent class="max-h-80">
              <SelectItem value="system">{{ t("core.settings.languageSystem") }}</SelectItem>
              <SelectItem v-for="option in locales" :key="option.value" :value="option.value">
                {{ option.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </Field>
      </div>
    </SectionCard>

    <component :is="section" v-for="(section, index) in sections" :key="index" />

    <SectionCard :title="t('core.settings.about')">
      <div class="flex flex-col gap-5 text-sm">
        <div>
          <p class="text-muted-foreground mb-3">{{ t("core.settings.authors") }}</p>
          <div class="flex flex-wrap gap-2">
            <Button
              v-for="author in AUTHORS"
              :key="author.name"
              variant="secondary"
              @click="openExternal(author.url)"
            >
              {{ author.name }}
            </Button>
          </div>
        </div>
        <div class="flex flex-wrap items-center justify-between gap-2">
          <span class="text-muted-foreground">{{ t("core.settings.license") }}: MIT</span>
          <Button variant="outline" @click="openExternal(REPOSITORY_URL)">
            <ExternalLink />
            {{ t("core.settings.repository") }}
          </Button>
        </div>
        <div>
          <p class="text-muted-foreground mb-1">{{ t("core.settings.acknowledgements") }}</p>
          <ul class="-mx-2 flex flex-col">
            <li v-for="entry in ACKNOWLEDGEMENTS" :key="entry.key">
              <button
                type="button"
                class="state-layer relative flex min-h-12 w-full items-center gap-3 rounded-md px-2 py-3 text-start"
                @click="openExternal(entry.url)"
              >
                <span class="flex-1">{{ t(entry.key) }}</span>
                <ExternalLink class="text-muted-foreground size-4.5 shrink-0" />
              </button>
            </li>
          </ul>
        </div>
      </div>
    </SectionCard>
  </div>
</template>
