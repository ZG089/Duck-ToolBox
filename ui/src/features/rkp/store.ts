import { defineStore } from "pinia"
import { ref } from "vue"

import type { Info, Keybox, Provision, Verify } from "./api"
import { rkpApi } from "./api"
import type { Draft } from "./draft"
import { applyDetected, hasGenericDevice, toDraft, toProfile } from "./draft"

/** Profile draft and the latest results, shared by every RKP tab. */
export const useRkpStore = defineStore("rkp", () => {
  const draft = ref<Draft | null>(null)
  const secretsPath = ref("")
  const info = ref<Info | null>(null)
  const provision = ref<Provision | null>(null)
  const keybox = ref<Keybox | null>(null)
  const verify = ref<Verify | null>(null)
  const verifyPath = ref("")

  async function load() {
    const shown = await rkpApi.show()
    secretsPath.value = shown.paths.profile_secrets_path
    let next = toDraft(shown.profile)
    if (hasGenericDevice(next)) {
      next = applyDetected(next, (await rkpApi.detect()).profile)
    }
    draft.value = next
  }

  async function save() {
    if (!draft.value) return
    const saved = await rkpApi.save(toProfile(draft.value))
    secretsPath.value = saved.paths.profile_secrets_path
    draft.value = toDraft(saved.profile)
  }

  async function detect() {
    if (!draft.value) return
    draft.value = applyDetected(draft.value, (await rkpApi.detect()).profile)
  }

  async function clear() {
    await rkpApi.clear()
    info.value = provision.value = keybox.value = verify.value = null
    await load()
  }

  /** Every action runs against the saved profile, so unsaved edits are saved first. */
  async function run<T>(action: () => Promise<T>) {
    await save()
    return action()
  }

  return {
    draft,
    secretsPath,
    info,
    provision,
    keybox,
    verify,
    verifyPath,
    load,
    save,
    detect,
    clear,
    checkKey: async () => (info.value = await run(rkpApi.info)),
    runProvision: async () => {
      provision.value = await run(rkpApi.provision)
      verifyPath.value = provision.value.csr_path
    },
    runKeybox: async () => {
      keybox.value = await run(rkpApi.keybox)
      verifyPath.value = keybox.value.csr_path
    },
    runVerify: async () => (verify.value = await rkpApi.verify(verifyPath.value.trim())),
  }
})
