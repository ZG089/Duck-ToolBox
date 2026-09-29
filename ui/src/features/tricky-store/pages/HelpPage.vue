<script setup lang="ts">
import { computed } from "vue"

import MarkdownView from "@/core/components/MarkdownView.vue"
import SectionCard from "@/core/components/SectionCard.vue"

import { useTrickyI18n } from "../i18n"
import type messages from "../locales/en.json"
import { repairMarkdown } from "../translations"

const { t } = useTrickyI18n()
const ta = (key: keyof (typeof messages)["ta"]) => repairMarkdown(t(`ta.${key}`))

/** Tricky Addon's instructions, in its own words and translations. */
const source = computed(() =>
  [
    `### ${ta("functional_button_save")}`,
    ta("help_save_and_update_description"),
    `### ${ta("help_select_denylist")}`,
    ta("help_select_denylist_description"),
    `### ${ta("help_deselect_unnecessary")}`,
    ta("help_deselect_unnecessary_description"),
    `### ${ta("help_add_system_app")}`,
    ta("help_add_system_app_description"),
    `### ${ta("help_set_keybox")}`,
    ta("help_set_keybox_description"),
    [
      ta("help_set_keybox_aosp"),
      ta("help_set_keybox_unknown"),
      ta("help_set_keybox_local"),
      ta("help_set_keybox_custom"),
    ]
      .map((line) => `- ${line}`)
      .join("\n"),
    `### ${ta("help_set_default_policy")}`,
    ta("help_set_default_policy_description"),
    `### ${ta("help_prop_settings")}`,
    ta("help_prop_settings_description"),
    [ta("help_prop_settings_handler"), ta("help_prop_settings_boot_hash")]
      .map((line) => `- ${line}`)
      .join("\n"),
    `### ${t("help.modes")}`,
    t("help.modesDescription"),
    `### ${t("help.backends")}`,
    t("help.backendsDescription"),
    `> ${ta("about_disclaimer")}`,
  ].join("\n\n"),
)
</script>

<template>
  <div class="flex flex-col gap-6 p-4">
    <SectionCard :title="ta('help_help_instructions')">
      <MarkdownView :source="source" />
    </SectionCard>
  </div>
</template>
