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
  <div class="flex flex-col gap-4 p-4">
    <SectionCard :title="t('core.settings.appearance')">
      <div class="flex flex-col gap-5">
        <Field>
          <FieldLabel>{{ t("core.settings.theme") }}</FieldLabel>
          <ToggleGroup
            type="single"
            variant="outline"
            class="w-full"
            :model-value="theme"
            @update:model-value="setTheme"
          >
            <ToggleGroupItem
              v-for="option in themes"
              :key="option.value"
              :value="option.value"
              class="flex-1"
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
      <div class="flex flex-col gap-4 text-sm">
        <div>
          <p class="text-muted-foreground mb-2">{{ t("core.settings.authors") }}</p>
          <div class="flex flex-wrap gap-2">
            <Button
              v-for="author in AUTHORS"
              :key="author.name"
              variant="secondary"
              size="sm"
              @click="openExternal(author.url)"
            >
              {{ author.name }}
            </Button>
          </div>
        </div>
        <div class="flex flex-wrap items-center justify-between gap-2">
          <span class="text-muted-foreground">{{ t("core.settings.license") }}: MIT</span>
          <Button variant="outline" size="sm" @click="openExternal(REPOSITORY_URL)">
            <ExternalLink />
            {{ t("core.settings.repository") }}
          </Button>
        </div>
        <div>
          <p class="text-muted-foreground mb-2">{{ t("core.settings.acknowledgements") }}</p>
          <ul class="flex flex-col gap-2">
            <li v-for="entry in ACKNOWLEDGEMENTS" :key="entry.key">
              <button
                type="button"
                class="text-start underline-offset-4 hover:underline"
                @click="openExternal(entry.url)"
              >
                {{ t(entry.key) }}
              </button>
            </li>
          </ul>
        </div>
      </div>
    </SectionCard>
  </div>
</template>
