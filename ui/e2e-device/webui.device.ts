import { expect, test } from "./fixtures"

test.describe.configure({ mode: "serial" })

test("runs inside the KernelSU manager with its APIs and insets", async ({ page, device }) => {
  const used = ["exec", "moduleInfo", "listPackages", "getPackagesInfo", "toast", "exit"]
  const environment = await page.evaluate((names) => {
    const ksu = (window as unknown as { ksu?: Record<string, unknown> }).ksu
    return {
      missing: names.filter((name) => typeof ksu?.[name] !== "function"),
      inset: getComputedStyle(document.documentElement).getPropertyValue("--window-inset-top"),
      chrome: Number(/Chrome\/(\d+)/.exec(navigator.userAgent)?.[1]),
    }
  }, used)
  expect(environment.missing).toEqual([])
  expect(environment.inset).not.toBe("")
  expect(environment.chrome).toBeGreaterThanOrEqual(111)

  await expect(page.getByText(/KernelSU ksud \d/)).toBeVisible()
  for (const tool of ["Tricky Store", "RKP Workbench", "Device ID Provisioner"]) {
    await expect(page.getByRole("button", { name: new RegExp(tool) })).toBeEnabled()
  }
  await device.screenshot("home")
})

test("edits the real Tricky Store target list", async ({ page, device }) => {
  await page.evaluate(() => (location.hash = "#/tricky-store"))
  await expect(page.getByText("Tricky Store (legacy)")).toBeVisible()

  const detector = page.getByRole("checkbox", { name: /Duck Detector/ })
  if ((await detector.getAttribute("aria-checked")) === "true") await detector.click()
  await detector.click()
  await page
    .getByRole("button", { name: "Mode and policy of io.github.vvb2060.keyattestation" })
    .click()
  await page.getByRole("dialog").getByLabel("Leaf Hack").check()
  await page.getByRole("dialog").getByRole("button", { name: "Save" }).click()
  await page.getByRole("button", { name: /^Save/ }).last().click()
  await expect(page.getByText("Config saved")).toBeVisible()

  const targets = await device.shell("cat /data/adb/tricky_store/target.txt")
  expect(targets).toContain("com.eltavine.duckdetector")
  expect(targets).toContain("io.github.vvb2060.keyattestation?")
  await device.screenshot("tricky-store")
})

test("closes a sheet with the Android back key", async ({ page, device }) => {
  await page.evaluate(() => (location.hash = "#/tricky-store"))
  await page.getByRole("button", { name: /Mode and policy of Google Play services/ }).click()
  await expect(page.getByRole("dialog")).toBeVisible()
  await device.shell("input keyevent KEYCODE_BACK")
  await expect(page.getByRole("dialog")).toBeHidden()
  await expect(page.getByText("Tricky Store (legacy)")).toBeVisible()
})

test("installs keyboxes into the running Tricky Store", async ({ page, device }) => {
  await page.evaluate(() => (location.hash = "#/tricky-store/keybox"))
  await page.getByRole("button", { name: /Unknown/ }).click()
  await expect(page.getByText("Unknown keybox set successfully")).toBeVisible()
  expect(
    await device.shell("grep -c BEGIN.CERTIFICATE /data/adb/tricky_store/keybox.xml"),
  ).toContain("4")

  await page.getByRole("button", { name: /AOSP/ }).click()
  await expect(page.getByText("AOSP keybox set successfully")).toBeVisible()
  expect(await device.shell("ls /data/adb/tricky_store")).toContain("keybox.xml.bak")
  await device.screenshot("keybox")
})

test("writes the legacy security patch file and prop settings", async ({ page, device }) => {
  await page.evaluate(() => (location.hash = "#/tricky-store/policy"))
  await page.getByLabel("System patch").fill("202609")
  await page.getByRole("button", { name: "Save" }).click()
  await expect(page.getByText("Default policy saved")).toBeVisible()
  expect(await device.shell("cat /data/adb/tricky_store/security_patch.txt")).toContain("202609")

  await page.evaluate(() => (location.hash = "#/tricky-store/props"))
  await page.getByLabel("Boot Hash").fill("ab".repeat(32))
  await page.getByRole("button", { name: "Save" }).click()
  await expect(page.getByText("Verified Boot Hash saved successfully")).toBeVisible()
  expect((await device.shell("cat /data/adb/boot_hash")).trim()).toBe("ab".repeat(32))
})

test("gives Tricky Store its own WebUI entry that opens this manager", async ({ page, device }) => {
  await page.evaluate(() => (location.hash = "#/settings"))
  const entry = page.getByRole("switch", { name: "Keystore module entry" })
  if ((await entry.getAttribute("aria-checked")) !== "true") await entry.click()
  await expect(entry).toBeChecked()
  expect(await device.shell("readlink /data/adb/modules/tricky_store/webroot")).toContain(
    "/data/adb/modules/duck-toolbox/webroot",
  )

  const hosted = await device.openWebUI("tricky_store")
  await expect(hosted).toHaveURL(/#\/tricky-store$/)
  await expect(hosted.getByText("Tricky Store (legacy)")).toBeVisible()
  await device.screenshot("entry")
})

test("checks for updates and lists the command history", async ({ page, device }) => {
  await page.evaluate(() => (location.hash = "#/system"))
  // The stable channel reads updateJson from GitHub.
  await expect(page.getByText("You are on the latest version.")).toBeVisible()
  await device.screenshot("system")
  await page.getByRole("button", { name: "Show all" }).click()
  await expect(page.getByText("tricky-store.save").first()).toBeVisible()
})
