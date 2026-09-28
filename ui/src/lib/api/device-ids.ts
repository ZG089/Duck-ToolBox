import { execJson } from "@/lib/api/client"
import type { Envelope } from "@/lib/api/client"
import type { ArtifactsData, DeviceIdsProfileData, DeviceIdsProvisionData } from "@/lib/types"

export function deviceIdsDefaultsCommand(): Promise<Envelope<DeviceIdsProfileData>> {
  return execJson(["device-ids", "defaults", "--json"])
}

export function deviceIdsProvisionCommand(
  profile: DeviceIdsProfileData,
): Promise<Envelope<DeviceIdsProvisionData>> {
  return execJson(["device-ids", "provision", "--stdin-json", "--json"], profile)
}

export function artifactsCommand(): Promise<Envelope<ArtifactsData>> {
  return execJson(["artifacts", "list", "--json"])
}
