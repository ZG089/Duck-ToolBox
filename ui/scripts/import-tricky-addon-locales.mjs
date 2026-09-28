#!/usr/bin/env node
// Imports Tricky Addon's translations (Android-style `strings/<locale>.xml`) into the
// `ta` group of the Tricky Store feature's locale files, keeping every other group, and
// records the bundle version (`locales/version` upstream) the in-app updater compares to.
//
//   pnpm locales:import <Tricky-Addon checkout>/webui/public/locales

import { readdir, readFile, writeFile } from "node:fs/promises"
import path from "node:path"
import { fileURLToPath } from "node:url"

import { XMLParser } from "fast-xml-parser"

import { pick } from "../src/features/tricky-store/translations.ts"

const feature = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "../src/features/tricky-store",
)

export { convert } from "../src/features/tricky-store/translations.ts"

async function main(source) {
  if (!source) throw new Error("usage: import-tricky-addon-locales <locales directory>")
  const parser = new XMLParser({
    ignoreAttributes: false,
    attributeNamePrefix: "",
    isArray: (name) => name === "string",
    textNodeName: "text",
    trimValues: false,
    parseTagValue: false,
  })
  const strings = path.join(source, "strings")
  const files = (await readdir(strings)).filter((name) => name.endsWith(".xml"))
  for (const file of files.sort()) {
    const locale = file.replace(/\.xml$/, "")
    const xml = parser.parse(await readFile(path.join(strings, file), "utf8"))
    const entries = new Map(
      (xml.resources?.string ?? []).map((entry) => [entry.name, String(entry.text ?? "")]),
    )
    const ta = pick(entries)

    const destination = path.join(feature, "locales", `${locale}.json`)
    const existing = await readFile(destination, "utf8").then(JSON.parse, () => ({}))
    await writeFile(destination, `${JSON.stringify({ ...existing, ta }, null, 2)}\n`)
    console.log(`${locale}: ${Object.keys(ta).length} strings`)
  }

  const version = (await readFile(path.join(source, "version"), "utf8")).trim()
  await writeFile(
    path.join(feature, "tricky-addon.json"),
    `${JSON.stringify({ version }, null, 2)}\n`,
  )
  console.log(`bundle version ${version}`)
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main(process.argv[2]).catch((error) => {
    console.error(error.message)
    process.exit(1)
  })
}
