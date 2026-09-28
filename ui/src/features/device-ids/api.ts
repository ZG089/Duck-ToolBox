import { z } from "zod"

import { duckd } from "@/core/duckd"

const profileSchema = z.object({
  brand: z.string(),
  device: z.string(),
  product: z.string(),
  serial: z.string(),
  manufacturer: z.string(),
  model: z.string(),
  imei: z.string(),
  imei2: z.string(),
  meid: z.string(),
  meid2: z.string(),
  ta_name: z.string(),
  ta_path: z.string(),
  dry_run: z.boolean(),
})
export type DeviceIdsProfile = z.infer<typeof profileSchema>

const resultSchema = z.object({
  count: z.number(),
  ids: z.array(z.object({ label: z.string(), value: z.string() })),
  dry_run: z.boolean(),
  ta_name: z.string(),
  ta_path: z.string(),
  loaded_library: z.string().nullable(),
  ta_api_version: z.string().nullable(),
  ta_version: z.string().nullable(),
  report_path: z.string(),
})
export type DeviceIdsResult = z.infer<typeof resultSchema>

export const deviceIdsApi = {
  defaults: () => duckd(["device-ids", "defaults"], { schema: profileSchema }),
  provision: (profile: DeviceIdsProfile) =>
    duckd(["device-ids", "provision"], { input: profile, schema: resultSchema }),
}
