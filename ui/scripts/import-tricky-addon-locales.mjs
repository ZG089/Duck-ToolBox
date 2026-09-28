#!/usr/bin/env node
// Imports Tricky Addon's translations (Android-style `strings/<locale>.xml`) into the
// `ta` group of the Tricky Store feature's locale files, keeping every other group.
//
//   pnpm locales:import <Tricky-Addon checkout>/webui/public/locales/strings

import { readdir, readFile, writeFile } from "node:fs/promises"
import path from "node:path"
import { fileURLToPath } from "node:url"

import { XMLParser } from "fast-xml-parser"

const KEYS = [
  "help_help_instructions",
  "help_save_and_update_description",
  "help_select_denylist",
  "help_select_denylist_description",
  "help_deselect_unnecessary",
  "help_deselect_unnecessary_description",
  "help_add_system_app",
  "help_add_system_app_description",
  "help_set_keybox",
  "help_set_keybox_description",
  "help_set_keybox_aosp",
  "help_set_keybox_unknown",
  "help_set_keybox_local",
  "help_set_keybox_custom",
  "help_set_default_policy",
  "help_set_default_policy_description",
  "help_prop_settings",
  "help_prop_settings_description",
  "help_prop_settings_handler",
  "help_prop_settings_boot_hash",
  "search_bar_search_placeholder",
  "functional_button_save",
  "functional_button_reset",
  "functional_button_remove",
  "functional_button_today",
  "menu_select_denylist",
  "menu_deselect_unnecessary",
  "menu_add_system_app",
  "menu_keybox",
  "menu_keybox_aosp",
  "menu_keybox_unknown",
  "menu_keybox_local",
  "menu_keybox_repo",
  "menu_prop_setting",
  "menu_set_default_policy",
  "menu_help",
  "prop_handler",
  "boot_hash_title",
  "about_disclaimer",
  "prompt_aosp_key_set",
  "prompt_key_set_error",
  "prompt_unknown_key_set",
  "prompt_boot_hash_set",
  "prompt_boot_hash_set_error",
  "prompt_saved_target",
  "prompt_save_error",
  "prompt_custom_key_set",
  "prompt_custom_key_set_error",
  "prompt_custom_saved",
  "prompt_custom_fetch_error",
  "prompt_custom_not_found",
  "prompt_custom_invalid_script",
  "prompt_custom_removed",
  "prompt_keybox_repo_set",
  "prompt_keybox_repo_set_error",
  "prompt_keybox_repo_download_error",
  "default_policy_title",
  "security_patch_invalid_all",
  "add_system_app_title",
  "customkb_dialog_title",
  "customkb_name_placeholder",
  "customkb_script_placeholder",
  "customkb_export_success",
  "customkb_export_error",
  "customkb_export_empty",
  "customkb_import_success",
  "customkb_import_error",
  "customkb_remove_title",
  "customkb_remove_message",
  "customkb_reset_message",
  "mode_dialog_title",
  "mode_auto",
  "mode_certificate_generating",
  "mode_leaf_hack",
  "mode_set_custom_policy",
  "mode_use_default_policy",
]

const target = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../src/features/tricky-store/locales",
)

/** Android format strings to vue-i18n list interpolation, with message syntax escaped. */
export function convert(value) {
  let next = 0
  return value
    .replace(/\\n/g, "\n")
    .replace(/\\(['"])/g, "$1")
    .replace(/[{}@|]|%(?:(\d+)\$)?([%sdfx])/g, (match, position, type) => {
      if (!type) return `{'${match}'}`
      if (type === "%") return "%"
      const index = position ? Number(position) - 1 : next++
      return `{${index}}`
    })
}

async function main(source) {
  if (!source) throw new Error("usage: import-tricky-addon-locales <strings directory>")
  const parser = new XMLParser({
    ignoreAttributes: false,
    attributeNamePrefix: "",
    isArray: (name) => name === "string",
    textNodeName: "text",
    trimValues: false,
    parseTagValue: false,
  })
  const files = (await readdir(source)).filter((name) => name.endsWith(".xml"))
  for (const file of files.sort()) {
    const locale = file.replace(/\.xml$/, "")
    const xml = parser.parse(await readFile(path.join(source, file), "utf8"))
    const strings = new Map(
      (xml.resources?.string ?? []).map((entry) => [entry.name, String(entry.text ?? "")]),
    )
    const ta = {}
    for (const key of KEYS) {
      const value = strings.get(key)?.trim()
      if (value) ta[key] = convert(value)
    }

    const destination = path.join(target, `${locale}.json`)
    const existing = await readFile(destination, "utf8").then(JSON.parse, () => ({}))
    const merged = { ...existing, ta }
    await writeFile(destination, `${JSON.stringify(merged, null, 2)}\n`)
    console.log(`${locale}: ${Object.keys(ta).length}/${KEYS.length}`)
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main(process.argv[2]).catch((error) => {
    console.error(error.message)
    process.exit(1)
  })
}
