/**
 * Compatibility facade. The API is split into focused modules under `lib/api/`; this file
 * re-exports the pieces the RKP and Device ID workbenches consume so those features did not
 * need to change when the backend envelope evolved.
 */
import { bridgeStatus, copyToClipboard, pushToast } from "@/lib/api/client"
import type { Envelope } from "@/lib/api/client"
import { installGeneratedKeybox } from "@/lib/api/tricky-store"
import type { KeyboxInstallResult } from "@/features/tricky-store/types"
import { translate } from "@/i18n"
import type { CommandHistoryEntry } from "@/lib/types"

export { bridgeStatus, pushToast, copyToClipboard }
export {
  artifactsCommand,
  deviceIdsDefaultsCommand,
  deviceIdsProvisionCommand,
} from "@/lib/api/device-ids"
export {
  infoCommand,
  keyboxCommand,
  profileClear,
  profileSave,
  profileShow,
  provisionCommand,
  systemProfileDefaults,
  verifyCommand,
} from "@/lib/api/rkp"

export function historyEntry<T>(envelope: Envelope<T>): CommandHistoryEntry {
  return {
    command: envelope.command,
    ok: envelope.ok,
    at: new Date().toISOString(),
    message: envelope.ok
      ? translate("messages.commandCompleted")
      : (envelope.error?.message ?? translate("messages.commandFailed")),
  }
}

/** Installs an RKP-generated keybox into the active keystore backend. */
export function replaceTrickyStoreKeyboxCommand(
  sourcePath: string,
): Promise<Envelope<KeyboxInstallResult>> {
  return installGeneratedKeybox(sourcePath)
}
