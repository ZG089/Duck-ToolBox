import { useMutation, useQueryClient } from "@tanstack/vue-query"

import { notifySuccess } from "@/core/notify"

import type { KeyboxInstall } from "./api"
import { useTrickyI18n } from "./i18n"
import { STATUS_KEY } from "./queries"

type Messages = Parameters<ReturnType<typeof useTrickyI18n>["t"]>[0]

/**
 * Installing a keybox from any source: Tricky Addon's success and failure messages, the
 * written path, and a refreshed keybox status.
 */
export function useKeyboxInstall<Input>(
  run: (input: Input) => Promise<KeyboxInstall>,
  messages: { success: Messages; failure?: Messages },
) {
  const { t } = useTrickyI18n()
  const client = useQueryClient()
  return useMutation({
    mutationFn: run,
    meta: { errorTitle: () => t(messages.failure ?? "ta.prompt_key_set_error") },
    onSuccess: (result) => {
      const detail = [
        t("keybox.installedDetail", { count: result.summary.keyboxes, path: result.target_path }),
        result.backup_path ? t("keybox.backup", { path: result.backup_path }) : "",
      ]
      notifySuccess(t(messages.success), detail.filter(Boolean).join("\n"))
      void client.invalidateQueries({ queryKey: STATUS_KEY })
    },
  })
}
