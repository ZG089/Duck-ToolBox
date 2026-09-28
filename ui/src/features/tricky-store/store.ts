import { defineStore } from "pinia"
import { computed, ref, shallowRef } from "vue"

import type { Policy, Status, TargetMode } from "./api"
import { trickyStoreApi } from "./api"
import type { Draft } from "./selection"
import { deselect, draftFromStatus, draftKey, select, toSaveRequest } from "./selection"

/**
 * Unsaved edits of the target list. Like Tricky Addon, taps only change this draft; the
 * Save button writes it to the active keystore module.
 */
export const useTrickyStore = defineStore("tricky-store", () => {
  const status = shallowRef<Status | null>(null)
  const draft = ref<Draft | null>(null)

  const baseline = computed(() => (status.value ? draftKey(draftFromStatus(status.value)) : ""))
  const dirty = computed(() => !!draft.value && draftKey(draft.value) !== baseline.value)
  const schema = computed(() => status.value?.schema ?? null)

  /** Takes a fresh status; pending edits survive unless `discard` is set. */
  function load(next: Status, discard = false) {
    const keep = !discard && dirty.value
    status.value = next
    if (!keep || !draft.value) draft.value = draftFromStatus(next)
  }

  function toggle(packageName: string) {
    const targets = draft.value!.targets
    if (targets.has(packageName)) targets.delete(packageName)
    else targets.set(packageName, "auto")
  }

  function setMode(packageName: string, mode: TargetMode, policy: Policy | null) {
    draft.value!.targets.set(packageName, mode)
    if (policy) draft.value!.perApp.set(packageName, policy)
    else draft.value!.perApp.delete(packageName)
  }

  const selectMany = (packages: Iterable<string>) => select(draft.value!, packages)
  const deselectMany = (packages: Iterable<string>) => deselect(draft.value!, packages)

  /** Checked system apps become targets and unchecked ones stop being targets, as upstream. */
  function setSystemApps(checked: ReadonlySet<string>) {
    const current = draft.value!
    for (const entry of status.value?.packages ?? []) {
      if (!entry.system) continue
      if (checked.has(entry.package_name)) select(current, [entry.package_name])
      else deselect(current, [entry.package_name])
    }
    current.systemApps = new Set(checked)
  }

  async function save() {
    await trickyStoreApi.save(toSaveRequest(draft.value!))
  }

  /** Saves one setting on top of the saved config, leaving other pending edits pending. */
  async function saveOnly(patch: Partial<Draft>) {
    const saved = draftFromStatus(status.value!)
    await trickyStoreApi.save(toSaveRequest({ ...saved, ...patch }))
    if (draft.value) Object.assign(draft.value, patch)
  }

  return {
    status,
    draft,
    dirty,
    schema,
    load,
    toggle,
    setMode,
    selectMany,
    deselectMany,
    setSystemApps,
    save,
    saveOnly,
  }
})
