import { reactive } from "vue"
import { toast } from "vue-sonner"

import { isDuckError } from "@/core/duckd"
import { findFeature } from "@/core/features"
import { i18n } from "@/core/i18n"

/** The error whose details dialog is open, shared by the whole app. */
export const errorDetails = reactive<{ error: unknown; open: boolean }>({
  error: null,
  open: false,
})

export function showErrorDetails(error: unknown): void {
  errorDetails.error = error
  errorDetails.open = true
}

/**
 * A localized one-line summary. Backend codes are looked up in the issuing feature's
 * `errors` messages first, then in the shared `core.errors`.
 */
export function errorTitle(error: unknown): string {
  const { t, te } = i18n.global
  if (isDuckError(error)) {
    const namespace = findFeature(error.command.split(".")[0] ?? "")?.namespace
    const featureKey = namespace ? `${namespace}.errors.${error.code}` : ""
    if (featureKey && te(featureKey, "en")) return t(featureKey)
    if (te(`core.errors.${error.code}`, "en")) return t(`core.errors.${error.code}`)
  }
  return t("core.errors.title")
}

export function errorDescription(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

export function notifyError(error: unknown, title: string = errorTitle(error)): void {
  toast.error(title, {
    description: errorDescription(error),
    action: {
      label: i18n.global.t("core.actions.details"),
      onClick: () => showErrorDetails(error),
    },
  })
}

export function notifySuccess(message: string, description?: string): void {
  toast.success(message, description ? { description } : undefined)
}
