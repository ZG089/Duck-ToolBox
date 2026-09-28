<script setup lang="ts">
import { computed } from "vue"

import MarkdownView from "@/core/components/MarkdownView.vue"
import SectionCard from "@/core/components/SectionCard.vue"

import { useTrickyI18n } from "../i18n"

const { t } = useTrickyI18n()

/** Tricky Addon's instructions, in its own words and translations. */
const source = computed(() =>
  [
    `### ${t("ta.functional_button_save")}`,
    t("ta.help_save_and_update_description"),
    `### ${t("ta.help_select_denylist")}`,
    t("ta.help_select_denylist_description"),
    `### ${t("ta.help_deselect_unnecessary")}`,
    t("ta.help_deselect_unnecessary_description"),
    `### ${t("ta.help_add_system_app")}`,
    t("ta.help_add_system_app_description"),
    `### ${t("ta.help_set_keybox")}`,
    t("ta.help_set_keybox_description"),
    [
      t("ta.help_set_keybox_aosp"),
      t("ta.help_set_keybox_unknown"),
      t("ta.help_set_keybox_local"),
      t("ta.help_set_keybox_custom"),
    ]
      .map((line) => `- ${line}`)
      .join("\n"),
    `### ${t("ta.help_set_default_policy")}`,
    t("ta.help_set_default_policy_description"),
    `### ${t("ta.help_prop_settings")}`,
    t("ta.help_prop_settings_description"),
    [t("ta.help_prop_settings_handler"), t("ta.help_prop_settings_boot_hash")]
      .map((line) => `- ${line}`)
      .join("\n"),
    `### ${t("help.modes")}`,
    t("help.modesDescription"),
    `### ${t("help.backends")}`,
    t("help.backendsDescription"),
    `> ${t("ta.about_disclaimer")}`,
  ].join("\n\n"),
)
</script>

<template>
  <div class="flex flex-col gap-4 p-4">
    <SectionCard :title="t('ta.help_help_instructions')">
      <MarkdownView :source="source" />
    </SectionCard>
  </div>
</template>
