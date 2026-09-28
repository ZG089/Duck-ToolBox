import { useClipboard } from "@vueuse/core"
import { toast } from "vue-sonner"

import { bridge } from "@/core/bridge"
import { duckd } from "@/core/duckd"
import { i18n } from "@/core/i18n"

/**
 * Opens a link in the system browser. The manager's WebView cannot leave the app, so this
 * goes through `am start` (`duckd system open-url`) and falls back to `window.open`.
 */
export async function openExternal(url: string): Promise<void> {
  if (bridge().available) {
    try {
      await duckd(["system", "open-url", url])
      return
    } catch {
      // Fall through to the browser.
    }
  }
  window.open(url, "_blank", "noopener")
}

export function useCopy() {
  // `legacy` falls back to execCommand when the WebView denies the async clipboard API.
  const clipboard = useClipboard({ legacy: true })
  return async (text: string) => {
    try {
      await clipboard.copy(text)
      toast.success(i18n.global.t("core.actions.copied"))
    } catch {
      toast.error(i18n.global.t("core.actions.copyFailed"))
    }
  }
}
