import { z } from "zod"

import { duckd } from "./client"

const manifestSchema = z.object({
  binary_version: z.string(),
  api: z.number(),
  features: z.array(z.object({ id: z.string(), summary: z.string(), contract: z.number() })),
})

export type FeatureManifest = z.infer<typeof manifestSchema>

/** Features compiled into the installed backend binary (`duckd features`). */
export function featureManifest(): Promise<FeatureManifest> {
  return duckd(["features"], { schema: manifestSchema })
}

const fileEntrySchema = z.object({
  name: z.string(),
  path: z.string(),
  directory: z.boolean(),
  size: z.number(),
  modified_unix: z.number(),
})

export const fileListSchema = z.object({
  path: z.string(),
  parent: z.string().nullable(),
  entries: z.array(fileEntrySchema),
})

export type FileEntry = z.infer<typeof fileEntrySchema>
export type FileList = z.infer<typeof fileListSchema>

/** Directory listing for file pickers (`duckd system files`). */
export function listFiles(path: string, extension = ""): Promise<FileList> {
  return duckd(["system", "files", "--path", path, "--extension", extension], {
    schema: fileListSchema,
  })
}
