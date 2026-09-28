import { computed } from "vue"
import { useRoute, useRouter } from "vue-router"

/**
 * A dialog or sheet whose open state lives in the URL (`?modal=<name>`). Opening pushes a
 * history entry, so the Android back gesture closes it the way KernelSU's WebUI activity
 * handles back: by calling `WebView.goBack()`.
 */
export function useRouteModal(name: string) {
  const route = useRoute()
  const router = useRouter()

  const isOpen = computed(() => route.query.modal === name)

  function show(params: Record<string, string> = {}) {
    if (isOpen.value) return
    void router.push({ query: { ...route.query, ...params, modal: name } })
  }

  function close() {
    if (!isOpen.value) return
    if (window.history.state?.back) {
      router.back()
    } else {
      const { modal: _modal, ...rest } = route.query
      void router.replace({ query: rest })
    }
  }

  const open = computed({
    get: () => isOpen.value,
    set: (value: boolean) => (value ? show() : close()),
  })

  function param(key: string): string | undefined {
    const value = route.query[key]
    return typeof value === "string" ? value : undefined
  }

  return { open, show, close, param }
}
