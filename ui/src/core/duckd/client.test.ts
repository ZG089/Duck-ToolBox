import { afterEach, describe, expect, it } from "vitest"
import { z } from "zod"

import type { RootBridge } from "@/core/bridge"
import { setBridge } from "@/core/bridge"

import { duckd } from "./client"
import { DuckError, parseEnvelope } from "./envelope"

function fakeBridge(stdout: string, host: RootBridge["hostModule"] = () => null) {
  const commands: string[] = []
  const bridge: RootBridge = {
    available: true,
    exec: async (command) => {
      commands.push(command)
      return { errno: 0, stdout, stderr: "" }
    },
    toast: () => undefined,
    hostModule: host,
    listPackages: async () => [],
    packagesInfo: async () => [],
    iconUrl: () => null,
    exit: () => undefined,
  }
  setBridge(bridge)
  return commands
}

const envelope = (fields: Record<string, unknown>) =>
  JSON.stringify({ ok: true, api: 1, command: "x", data: null, error: null, ts: 0, ...fields })

afterEach(() => setBridge(null))

describe("duckd", () => {
  it("quotes every argument and runs the module's wrapper", async () => {
    const commands = fakeBridge(envelope({ data: { ok: 1 } }))
    await duckd(["tricky-store", "keybox", "install", "/sdcard/it's.xml"])
    expect(commands).toEqual([
      `'/data/adb/modules/duck-toolbox/bin/duckctl.sh' 'tricky-store' 'keybox' 'install' '/sdcard/it'\\''s.xml'`,
    ])
  })

  it("uses the host module directory when opened from Duck ToolBox itself", async () => {
    const commands = fakeBridge(envelope({}), () => ({
      id: "duck-toolbox",
      dir: "/data/adb/modules_update/duck-toolbox",
    }))
    await duckd(["features"])
    expect(commands[0]).toMatch(/^'\/data\/adb\/modules_update\/duck-toolbox\/bin\/duckctl\.sh'/)
  })

  it("sends input as JSON through a quoted here-document", async () => {
    const commands = fakeBridge(envelope({}))
    await duckd(["tricky-store", "save"], { input: { note: "$(reboot)" } })
    const [first, payload] = commands[0]!.split("\n")
    expect(first).toMatch(/'--stdin-json' <<'DUCK_EOF_[A-Z0-9]+'$/)
    expect(JSON.parse(payload!)).toEqual({ note: "$(reboot)" })
  })

  it("maps error envelopes to DuckError with the backend code", async () => {
    fakeBridge(
      envelope({
        ok: false,
        command: "tricky-store.save",
        error: { code: "invalid_policy", message: "bad" },
      }),
    )
    const error = await duckd(["tricky-store", "save"]).catch((caught: unknown) => caught)
    expect(error).toBeInstanceOf(DuckError)
    expect(error).toMatchObject({ code: "invalid_policy", command: "tricky-store.save" })
  })

  it("rejects data that breaks the declared contract", async () => {
    fakeBridge(envelope({ data: { count: "three" } }))
    const error = await duckd(["x"], { schema: z.object({ count: z.number() }) }).catch(
      (caught: unknown) => caught,
    )
    expect(error).toMatchObject({ code: "contract_mismatch" })
  })

  it("rejects envelopes from a newer API", async () => {
    fakeBridge(envelope({ api: 2 }))
    await expect(duckd(["x"])).rejects.toMatchObject({ code: "api_mismatch" })
  })
})

describe("parseEnvelope", () => {
  it("takes the last envelope after warnings", () => {
    const stdout = `warning: something\n${envelope({ command: "a" })}\nnot json\n`
    expect(parseEnvelope(stdout, "", "a").command).toBe("a")
  })

  it("reports stderr when there is no envelope", () => {
    expect(() => parseEnvelope("", "permission denied", "a")).toThrow("permission denied")
  })
})
