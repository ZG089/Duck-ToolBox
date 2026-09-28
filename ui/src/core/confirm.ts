import { shallowRef } from "vue"

export interface ConfirmRequest {
  title: string
  description?: string
  confirmLabel?: string
  cancelLabel?: string
  destructive?: boolean
}

interface PendingConfirm extends ConfirmRequest {
  resolve(accepted: boolean): void
}

/** The confirmation currently on screen; rendered by the shell's confirm host. */
export const pendingConfirm = shallowRef<PendingConfirm | null>(null)

/** Asks the user to confirm; resolves `false` when dismissed. */
export function confirmAction(request: ConfirmRequest): Promise<boolean> {
  pendingConfirm.value?.resolve(false)
  return new Promise((resolve) => {
    pendingConfirm.value = {
      ...request,
      resolve: (accepted) => {
        pendingConfirm.value = null
        resolve(accepted)
      },
    }
  })
}
