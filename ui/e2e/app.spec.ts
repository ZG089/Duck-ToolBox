import { expect, test } from "./fixtures"

test("lists every tool on the home page", async ({ page, open }) => {
  await open("/")
  for (const name of [
    "Tricky Store",
    "RKP Workbench",
    "Device ID Provisioner",
    "Module & updates",
  ]) {
    await expect(page.getByRole("button", { name: new RegExp(name) })).toBeVisible()
  }
})

test("confirms the Device ID warning before opening the tool", async ({ page, open }) => {
  await open("/")
  await page.getByRole("button", { name: /Device ID Provisioner/ }).click()
  const dialog = page.getByRole("alertdialog")
  await expect(dialog.getByText(/Qualcomm-based OnePlus/)).toBeVisible()
  await dialog.getByRole("button", { name: "Continue" }).click()
  await expect(page.getByLabel("Brand")).toHaveValue("OnePlus")
})

test("runs a Device ID dry run", async ({ page, open }) => {
  await open("/device-ids")
  await expect(page.getByRole("switch", { name: "Dry run" })).toBeChecked()
  await page.getByRole("button", { name: "Run dry run" }).click()
  await expect(page.getByText("Device ID provisioning finished")).toBeVisible()
  await expect(page.getByText("device-ids-20260928.json")).toBeVisible()
})

test("saves an RKP profile with a direct seed", async ({ page, open }) => {
  await open("/rkp")
  await page.getByText("Direct seed").click()
  await page.getByLabel("Seed (CDI_Leaf)").fill("ab".repeat(32))
  await page.getByRole("button", { name: "Save profile" }).click()
  await expect(page.getByText("Profile saved")).toBeVisible()
})

test("installs an RKP keybox through the Tricky Store extension point", async ({ page, open }) => {
  await open("/rkp")
  await page.getByText("Direct seed").click()
  await page.getByLabel("Seed (CDI_Leaf)").fill("ab".repeat(32))
  await page.getByRole("tab", { name: "Keybox" }).click()
  await page.getByRole("button", { name: "Generate keybox" }).click()
  await expect(page.getByText("keybox.xml generated")).toBeVisible()
  await page.getByRole("button", { name: "Install to Tricky Store" }).click()
  await expect(
    page.getByText("Keybox installed to /data/adb/tricky_store/keybox.xml"),
  ).toBeVisible()
})

test("lays out right-to-left languages mirrored", async ({ page, open }) => {
  await open("/", { locale: "ar" })
  await expect(page.locator("html")).toHaveAttribute("dir", "rtl")
  await expect(page.locator("html")).toHaveAttribute("lang", "ar")
})

test("follows the saved language", async ({ page, open }) => {
  await open("/", { locale: "zh-CN" })
  await expect(page.getByText("工具")).toBeVisible()
  await expect(page.locator("html")).toHaveAttribute("dir", "ltr")
})

test("shows the entry toggle contributed by the Tricky Store feature", async ({ page, open }) => {
  await open("/settings")
  await expect(page.getByRole("switch", { name: "Keystore module entry" })).toBeChecked()
})
