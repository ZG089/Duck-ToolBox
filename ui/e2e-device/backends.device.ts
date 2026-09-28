import type { Page } from "@playwright/test"

import type { Device } from "./fixtures"
import { expect, test } from "./fixtures"

// Run by scripts/device-test.sh with only that keystore module enabled. Each daemon's own
// log is the acceptance check: it reloads the files the WebUI writes and says so.

async function ensureTarget(page: Page, name: RegExp) {
  await page.evaluate(() => (location.hash = "#/tricky-store"))
  const row = page.getByRole("checkbox", { name })
  if ((await row.getAttribute("aria-checked")) !== "true") await row.click()
  await page.getByRole("button", { name: /^Save/ }).last().click()
  await expect(page.getByText("Config saved")).toBeVisible()
}

async function installUnknownKeybox(page: Page) {
  await page.evaluate(() => (location.hash = "#/tricky-store/keybox"))
  await page.getByRole("button", { name: /Unknown/ }).click()
  await expect(page.getByText("Unknown keybox set successfully")).toBeVisible()
}

const log = (device: Device, tag: string) => device.shell(`logcat -d -v brief | grep '${tag}'`)

test("@teesim TEESimulator applies the target list, policy and a generated keybox", async ({
  page,
  device,
}) => {
  await device.shell("logcat -c")
  await ensureTarget(page, /Duck Detector/)
  await expect(page.getByText("TEESimulator", { exact: true })).toBeVisible()
  await expect(page.getByRole("button", { name: /Mode and policy of/ })).toHaveCount(0)

  await page.evaluate(() => (location.hash = "#/tricky-store/policy"))
  await page.getByRole("button", { name: "generation", exact: true }).click()
  await page.getByRole("button", { name: "Save" }).click()
  await expect(page.getByText("Default policy saved")).toBeVisible()
  await installUnknownKeybox(page)

  expect(await device.shell("cat /data/adb/teesim/config.json")).toContain(
    "com.eltavine.duckdetector",
  )
  await expect
    .poll(() => log(device, "TEESimulator"), { timeout: 60_000 })
    .toMatch(/staged profile 'default' \(mode=generation/)
  const lines = await log(device, "TEESimulator")
  expect(lines).toMatch(/keybox parsed/)
  expect(lines).toMatch(/ack epoch=\d+ ok=true applied=1 failed=0/)
  expect(lines).not.toMatch(/ok=false|failed=[1-9]/)
  await device.screenshot("teesim")
})

test("@oh_my_keymint OhMyKeymint reloads the scoop, trust settings and a generated keybox", async ({
  page,
  device,
}) => {
  await device.shell("logcat -c")
  await ensureTarget(page, /Duck Detector/)
  await expect(page.getByText("OhMyKeymint", { exact: true })).toBeVisible()

  await page.evaluate(() => (location.hash = "#/tricky-store/policy"))
  // OhMyKeymint only logs a security_patch that differs from the one it has.
  const patch = page.getByLabel("Security patch")
  await patch.fill((await patch.inputValue()) === "2026-09-01" ? "2026-08-01" : "2026-09-01")
  await page.getByRole("switch", { name: "Bootloader locked" }).click()
  await page.getByRole("button", { name: "Save" }).click()
  await expect(page.getByText("Default policy saved")).toBeVisible()
  // device_locked is applied by keymint only after a restart.
  await expect(
    page.getByText("Reboot to apply the changed Verified Boot and OS settings."),
  ).toBeVisible()
  await installUnknownKeybox(page)

  const injector = await device.shell("cat /data/misc/keystore/omk/injector.toml")
  expect(injector).toContain('"com.eltavine.duckdetector"')
  await expect
    .poll(() => log(device, "OhMyKeymint"), { timeout: 60_000 })
    .toMatch(/active keybox identity updated/)
  const lines = await log(device, "OhMyKeymint")
  expect(lines).toMatch(/reloaded config from \/data\/misc\/keystore\/omk\/injector.toml/)
  expect(lines).toMatch(/Applied runtime security_patch change/)
  expect(lines).not.toMatch(/failed to parse config|moved invalid config/)
  await device.screenshot("oh-my-keymint")
})
