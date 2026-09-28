import { mkdirSync } from "node:fs"

import type { AndroidDevice, Page } from "@playwright/test"
import { _android, test as base, expect } from "@playwright/test"

/**
 * Drives the WebUI inside the real KernelSU manager on a device prepared by
 * `scripts/device-test.sh`: KernelSU late-loaded, Duck ToolBox and Tricky Store installed,
 * WebView debugging on. Checks read the device's files back through adb.
 */
const MANAGER = "me.weishu.kernelsu"

export interface Device {
  adb: AndroidDevice
  /** Runs a shell command on the device (adbd runs as root). */
  shell(command: string): Promise<string>
  /** Opens the WebUI of a module through the manager's token-checked deep link. */
  openWebUI(moduleId: string): Promise<Page>
  /** Captures the whole device screen, status bar included. */
  screenshot(name: string): Promise<void>
}

const SCREENSHOTS = "test-results/device/screenshots"

export const test = base.extend<{ device: Device; page: Page }>({
  device: async ({}, use) => {
    const [adb] = await _android.devices()
    if (!adb) throw new Error("no adb device; run scripts/device-test.sh first")
    const token = process.env.DUCK_KSU_TOKEN
    if (!token) throw new Error("DUCK_KSU_TOKEN is not set; run scripts/device-test.sh first")
    const shell = async (command: string) => (await adb.shell(command)).toString("utf8")
    await use({
      adb,
      shell,
      screenshot: async (name) => {
        mkdirSync(SCREENSHOTS, { recursive: true })
        // The headless emulator composes frames in software; let the display catch up.
        await new Promise((resolve) => setTimeout(resolve, 2_000))
        await adb.screenshot({ path: `${SCREENSHOTS}/${name}.png` })
      },
      openWebUI: async (moduleId) => {
        await shell(`am force-stop ${MANAGER}`)
        // A cold manager start takes long on a busy emulator.
        const opened = adb.waitForEvent("webview", { timeout: 180_000 })
        await shell(
          `am start -a android.intent.action.VIEW -d 'ksu://webui?id=${moduleId}&token=${token}'`,
        )
        const page = await (await opened).page()
        await page.waitForLoadState("load")
        return page
      },
    })
    await adb.close()
  },
  page: async ({ device }, use) => {
    await use(await device.openWebUI("duck-toolbox"))
  },
})

export { expect }
