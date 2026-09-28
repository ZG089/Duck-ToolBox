// @vitest-environment node
import { lstatSync, readFileSync, readlinkSync } from "node:fs"
import path from "node:path"

import { afterEach, describe, expect, it } from "vitest"

import type { Device, Host, Server } from "@/core/testing/host"
import { CONTRACT, hasDuckd, installHost, serve } from "@/core/testing/host"

import { trickyStoreApi } from "./api"
import { draftFromStatus, select, toSaveRequest } from "./selection"

const AOSP_KEYBOX = readFileSync(
  path.resolve(import.meta.dirname, "../../../../crates/duck-tricky-store/assets/aosp-keybox.xml"),
  "utf8",
)

const moduleProp = (id: string, versionCode: number) =>
  `id=${id}\nname=${id}\nversion=v${versionCode}\nversionCode=${versionCode}\n`

const packages = {
  userPackages: ["com.google.android.apps.walletnfcrel", "io.github.vvb2060.keyattestation"],
  systemPackages: ["com.google.android.gms", "com.android.vending"],
}

const backends: Record<string, Device> = {
  "tricky-store": {
    files: {
      "/data/adb/modules/tricky_store/module.prop": moduleProp("tricky_store", 246),
      "/data/adb/tricky_store/config.ini": "[target]\nio.github.vvb2060.keyattestation!\n",
    },
  },
  "tricky-store-legacy": {
    files: {
      "/data/adb/modules/tricky_store/module.prop": moduleProp("tricky_store", 200),
      "/data/adb/tricky_store/target.txt": "io.github.vvb2060.keyattestation\n",
      "/data/adb/tricky_store/security_patch.txt": "system=prop\n",
    },
  },
  "tee-simulator": {
    files: {
      "/data/adb/modules/teesim/module.prop": moduleProp("teesim", 31),
      "/data/adb/teesim/keybox.xml": AOSP_KEYBOX,
      "/data/adb/teesim/config.json": JSON.stringify({
        version: 1,
        profiles: { default: { keybox: "keybox.xml", apps: ["io.github.vvb2060.keyattestation"] } },
      }),
    },
  },
  "oh-my-keymint": {
    files: {
      "/data/adb/modules/oh_my_keymint/module.prop": moduleProp("oh_my_keymint", 90),
      // Released OhMyKeymint (v1.2.0) has no version key here and refuses unknown keys.
      "/data/misc/keystore/omk/injector.toml": 'scoop = ["io.github.vvb2060.keyattestation"]\n',
      "/data/misc/keystore/omk/config.toml": 'version = 2\n[trust]\nsecurity_patch = "auto"\n',
    },
  },
}

let host: Host | undefined
let server: Server | undefined
afterEach(async () => {
  host?.dispose()
  await server?.close()
  server = undefined
})

describe.skipIf(!hasDuckd)("Tricky Store API against the real duckd", CONTRACT, () => {
  describe.each(Object.entries(backends))("%s", (backend, device) => {
    it("reads status and saves a changed target list", async () => {
      host = installHost({ ...device, ...packages })
      const status = await trickyStoreApi.status()
      expect(status.active?.backend).toBe(backend)
      expect(status.schema).not.toBeNull()
      expect(status.config.targets.map((target) => target.package_name)).toContain(
        "io.github.vvb2060.keyattestation",
      )

      const draft = draftFromStatus(status)
      select(draft, ["com.google.android.apps.walletnfcrel"])
      await trickyStoreApi.save(toSaveRequest(draft))
      const saved = await trickyStoreApi.status()
      expect(saved.config.targets.map((target) => target.package_name)).toContain(
        "com.google.android.apps.walletnfcrel",
      )
    })

    it("rejects a policy value the backend would not accept", async () => {
      host = installHost({ ...device, ...packages })
      const status = await trickyStoreApi.status()
      const field = status.schema!.default_policy.find((entry) => entry.kind === "text")!
      const request = toSaveRequest(draftFromStatus(status))
      request.default_policy = { [field.key]: "definitely not valid!" }
      await expect(trickyStoreApi.save(request)).rejects.toMatchObject({ code: "invalid_policy" })
    })

    it("installs keyboxes from every source", async () => {
      host = installHost({ ...device, ...packages })
      const aosp = await trickyStoreApi.keyboxAosp()
      expect(aosp.summary.keyboxes).toBe(1)
      const generated = await trickyStoreApi.keyboxGenerate()
      expect(generated.backup_path).not.toBeNull()
      const imported = await trickyStoreApi.keyboxImport(AOSP_KEYBOX)
      expect(imported.source).toBe("local")
      const local = await trickyStoreApi.keyboxInstall(aosp.target_path)
      expect(local.target_path).toBe(aosp.target_path)
      await expect(trickyStoreApi.keyboxImport("<not a keybox/>")).rejects.toMatchObject({
        code: "invalid_keybox",
      })
    })
  })

  it("tells when OhMyKeymint must restart and keeps its injector format", async () => {
    host = installHost({ ...backends["oh-my-keymint"], ...packages })
    const request = toSaveRequest(draftFromStatus(await trickyStoreApi.status()))
    const patchOnly = { ...request, default_policy: { security_patch: "2026-09-05" } }
    expect((await trickyStoreApi.save(patchOnly)).restart_required).toBe(false)
    const locked = { ...request, default_policy: { device_locked: "false" } }
    expect((await trickyStoreApi.save(locked)).restart_required).toBe(true)
    const injector = readFileSync(host.path("/data/misc/keystore/omk/injector.toml"), "utf8")
    expect(injector).not.toContain("version")
  })

  it("downloads keyboxes the way providers and the repository hand them over", async () => {
    server = await serve({
      "/plain.xml": AOSP_KEYBOX,
      "/encoded.txt": Buffer.from(AOSP_KEYBOX).toString("base64"),
    })
    host = installHost({ ...backends["tricky-store"], ...packages })
    expect((await trickyStoreApi.keyboxFetch(server.url("/plain.xml"))).source).toBe("url")
    const decoded = await trickyStoreApi.keyboxFetch(server.url("/encoded.txt"), "base64 -d")
    expect(decoded.summary.keyboxes).toBe(1)
    await expect(trickyStoreApi.keyboxFetch(server.url("/missing"))).rejects.toMatchObject({
      code: "download_failed",
    })
  })

  it("manages custom keybox providers", async () => {
    host = installHost({ ...backends["tricky-store"], ...packages })
    const providers = await trickyStoreApi.providers()
    const next = [
      ...providers,
      { name: "Mine", url: "https://example.org/kb", decode: "base64 -d" },
    ]
    expect(await trickyStoreApi.saveProviders(next)).toHaveLength(next.length)
    await expect(
      trickyStoreApi.saveProviders([{ name: "Bad", url: "https://example.org", decode: "sh" }]),
    ).rejects.toMatchObject({ code: "unsupported_decoder" })

    const { path: exported } = await trickyStoreApi.exportProviders()
    const content = readFileSync(host.path(exported), "utf8")
    await trickyStoreApi.resetProviders()
    const restored = await trickyStoreApi.importProvidersContent(content)
    expect(restored.map((provider) => provider.name)).toContain("Mine")
  })

  it("lists helper app sets and saves prop settings", async () => {
    host = installHost({ ...backends["tricky-store"], ...packages })
    expect((await trickyStoreApi.denylist()).packages).toEqual([])
    expect(Array.isArray((await trickyStoreApi.xposed()).packages)).toBe(true)
    expect((await trickyStoreApi.unnecessary()).packages.length).toBeGreaterThan(0)

    const hash = "ab".repeat(32)
    const props = await trickyStoreApi.saveProps({ prop_handler_enabled: false, boot_hash: hash })
    expect(props.boot_hash).toBe(hash)
    expect(readFileSync(host.path("/data/adb/boot_hash"), "utf8").trim()).toBe(hash)
    expect(lstatSync(host.path("/data/adb/disable_prop_handler")).isFile()).toBe(true)
  })

  it("links and unlinks the entry on the keystore module", async () => {
    host = installHost({ ...backends["tricky-store"], ...packages })
    expect((await trickyStoreApi.entry()).module_id).toBe("tricky_store")
    const enabled = await trickyStoreApi.setEntry(true)
    expect(enabled.linked).toBe(true)
    expect(readlinkSync(host.path("/data/adb/modules/tricky_store/webroot"))).toMatch(/webroot$/)
    expect((await trickyStoreApi.setEntry(false)).linked).toBe(false)
  })
})
