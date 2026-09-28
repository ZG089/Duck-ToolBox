import { execJson } from "@/lib/api/client"
import type { Envelope } from "@/lib/api/client"
import type {
  FileListData,
  KeyboxInstallResult,
  KeyboxProvider,
  PackageListData,
  Policy,
  PropSaveData,
  SaveRequest,
  SaveResult,
  TrickyStoreStatus,
} from "@/features/tricky-store/types"

export function statusCommand(): Promise<Envelope<TrickyStoreStatus>> {
  return execJson(["tricky-store", "status", "--json"])
}

export function saveCommand(request: SaveRequest): Promise<Envelope<SaveResult>> {
  return execJson(["tricky-store", "save", "--stdin-json", "--json"], request)
}

export function keyboxInstallCommand(sourcePath: string): Promise<Envelope<KeyboxInstallResult>> {
  return execJson(["tricky-store", "keybox", "install", sourcePath.trim(), "--json"])
}

export function keyboxSetAospCommand(): Promise<Envelope<KeyboxInstallResult>> {
  return execJson(["tricky-store", "keybox", "set-aosp", "--json"])
}

export function keyboxGenerateCommand(): Promise<Envelope<KeyboxInstallResult>> {
  return execJson(["tricky-store", "keybox", "generate", "--json"])
}

export function keyboxFetchCommand(url: string, decode: string): Promise<Envelope<KeyboxInstallResult>> {
  return execJson([
    "tricky-store",
    "keybox",
    "fetch",
    "--url",
    url.trim(),
    "--decode",
    decode.trim(),
    "--json",
  ])
}

export function providersListCommand(): Promise<Envelope<KeyboxProvider[]>> {
  return execJson(["tricky-store", "keybox", "providers", "list", "--json"])
}

export function providersSaveCommand(providers: KeyboxProvider[]): Promise<Envelope<KeyboxProvider[]>> {
  return execJson(["tricky-store", "keybox", "providers", "save", "--stdin-json", "--json"], providers)
}

export function providersResetCommand(): Promise<Envelope<KeyboxProvider[]>> {
  return execJson(["tricky-store", "keybox", "providers", "reset", "--json"])
}

export function providersImportCommand(path: string): Promise<Envelope<KeyboxProvider[]>> {
  return execJson(["tricky-store", "keybox", "providers", "import", path.trim(), "--json"])
}

export function providersExportCommand(): Promise<Envelope<{ path: string }>> {
  return execJson(["tricky-store", "keybox", "providers", "export", "--json"])
}

export function xposedCommand(): Promise<Envelope<PackageListData>> {
  return execJson(["tricky-store", "apps", "xposed", "--json"])
}

export function denylistCommand(): Promise<Envelope<PackageListData>> {
  return execJson(["tricky-store", "apps", "denylist", "--json"])
}

export function unnecessaryCommand(refresh: boolean): Promise<Envelope<PackageListData>> {
  const args = ["tricky-store", "apps", "unnecessary", "--json"]
  if (refresh) {
    args.splice(3, 0, "--refresh")
  }
  return execJson(args)
}

export function propsSaveCommand(
  propHandlerEnabled: boolean,
  bootHash: string | null,
): Promise<Envelope<PropSaveData>> {
  return execJson(["tricky-store", "props", "--stdin-json", "--json"], {
    prop_handler_enabled: propHandlerEnabled,
    boot_hash: bootHash,
  })
}

export function filesCommand(path: string, extension = "xml"): Promise<Envelope<FileListData>> {
  return execJson([
    "tricky-store",
    "files",
    "--path",
    path.trim() || "/storage/emulated/0/Download",
    "--extension",
    extension,
    "--json",
  ])
}

/** Installs a previously generated RKP keybox into the active backend. */
export function installGeneratedKeybox(sourcePath: string): Promise<Envelope<KeyboxInstallResult>> {
  return keyboxInstallCommand(sourcePath)
}

export type { Policy }
