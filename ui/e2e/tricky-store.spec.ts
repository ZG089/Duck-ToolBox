import { expect, test } from "./fixtures"

const row = (name: string) => ({ role: "checkbox" as const, name: new RegExp(name) })

test.describe("target list", () => {
  test("selects an app and saves", async ({ page, open }) => {
    await open("/tricky-store")
    await expect(page.getByText("4 selected")).toBeVisible()
    await page.getByRole(row("Discord").role, { name: row("Discord").name }).click()
    await expect(page.getByText("5 selected")).toBeVisible()
    await page.getByRole("button", { name: "Save", exact: false }).last().click()
    await expect(page.getByText("Config saved")).toBeVisible()
  })

  test("sets a per-app mode from the mode sheet", async ({ page, open }) => {
    await open("/tricky-store")
    await page.getByRole("button", { name: "Mode and policy of Key Attestation" }).click()
    const sheet = page.getByRole("dialog")
    await expect(sheet.getByText("Mode", { exact: true })).toBeVisible()
    await sheet.getByLabel("Leaf Hack").check()
    await sheet.getByRole("button", { name: "Save" }).click()
    await expect(sheet).toBeHidden()
    await expect(
      page.getByRole("checkbox", { name: /Key Attestation/ }).getByText("Leaf Hack"),
    ).toBeVisible()
  })

  test("closes the mode sheet with the back gesture", async ({ page, open }) => {
    await open("/tricky-store")
    await page.getByRole("button", { name: "Mode and policy of Google Wallet" }).click()
    await expect(page.getByRole("dialog")).toBeVisible()
    await page.goBack()
    await expect(page.getByRole("dialog")).toBeHidden()
    await expect(page).toHaveURL(/#\/tricky-store$/)
  })

  test("filters by label or package name", async ({ page, open }) => {
    await open("/tricky-store")
    await page.getByPlaceholder("Search").fill("wallet")
    await expect(page.getByRole("checkbox", { name: /Google Wallet/ })).toBeVisible()
    await expect(page.getByRole("checkbox", { name: /Discord/ })).toBeHidden()
  })

  test("select all, then deselect unnecessary apps", async ({ page, open }) => {
    await open("/tricky-store")
    await page.getByRole("button", { name: "More" }).click()
    await page.getByRole("menuitem", { name: "Select all", exact: true }).click()
    await page.getByRole("button", { name: "More" }).click()
    await page.getByRole("menuitem", { name: "Deselect Unnecessary" }).click()
    await expect(page.getByText("4 apps deselected")).toBeVisible()
    await expect(page.getByRole("checkbox", { name: /LSPosed/ })).toHaveAttribute(
      "aria-checked",
      "false",
    )
  })

  test("offers the DenyList only on Magisk", async ({ page, open }) => {
    await open("/tricky-store", { query: { manager: "magisk" } })
    await page.getByRole("button", { name: "More" }).click()
    await page.getByRole("menuitem", { name: "Select From DenyList" }).click()
    await expect(page.getByText("2 apps selected")).toBeVisible()
  })

  test("adds a system app", async ({ page, open }) => {
    await open("/tricky-store")
    await expect(page.getByRole("checkbox", { name: /Chrome/ })).toBeHidden()
    await page.getByRole("button", { name: "More" }).click()
    await page.getByRole("menuitem", { name: "Add System App" }).click()
    const dialog = page.getByRole("dialog")
    await dialog.getByRole("checkbox", { name: /Chrome/ }).click()
    await dialog.getByRole("button", { name: "Save" }).click()
    await expect(page.getByRole("checkbox", { name: /Chrome/ })).toHaveAttribute(
      "aria-checked",
      "true",
    )
  })
})

test.describe("keybox", () => {
  test("installs the AOSP keybox", async ({ page, open }) => {
    await open("/tricky-store/keybox")
    await page.getByRole("button", { name: /AOSP/ }).click()
    await expect(page.getByText("AOSP keybox set successfully")).toBeVisible()
  })

  test("rejects a provider with a disallowed decode step", async ({ page, open }) => {
    await open("/tricky-store/keybox")
    await page.getByRole("button", { name: "Add provider" }).click()
    const dialog = page.getByRole("dialog")
    await dialog.getByLabel("Name").fill("Mine")
    await dialog.getByLabel("URL").fill("https://example.org/kb")
    await dialog.getByLabel("Decode script").fill("sh -c reboot")
    await dialog.getByRole("button", { name: "Save" }).click()
    await expect(page.getByText("Decode step not allowed")).toBeVisible()
  })

  test("reports a failed provider download with Tricky Addon's message", async ({ page, open }) => {
    await open("/tricky-store/keybox")
    await page.getByRole("button", { name: "Add provider" }).click()
    const dialog = page.getByRole("dialog")
    await dialog.getByLabel("Name").fill("Broken")
    await dialog.getByLabel("URL").fill("https://example.org/fail")
    await dialog.getByRole("button", { name: "Save" }).click()
    await page.getByRole("button", { name: "Broken" }).click()
    await expect(page.getByText("Failed to fetch keybox")).toBeVisible()
  })
})

test.describe("keybox repository", () => {
  // A stand-in for keybox.kowx712.cc that speaks Tricky Addon's repo-api.md protocol.
  const site = (reply: string) => `<!doctype html><script>
    addEventListener("message", (event) => {
      if (event.data?.type !== "handshake") return
      event.source.postMessage({ type: "handshake_ack" }, event.origin)
      setTimeout(() => event.source.postMessage(${reply}, event.origin), 100)
    })
  </script>`

  test("installs the keybox the site hands over after the handshake", async ({ page, open }) => {
    await page.route("https://keybox.kowx712.cc/**", (route) =>
      route.fulfill({
        contentType: "text/html",
        body: site(`{ type: "download", url: "https://keybox.kowx712.cc/api/kb/1" }`),
      }),
    )
    await open("/tricky-store/keybox/repo")
    await expect(page.getByText("Keybox set from repo successfully")).toBeVisible()
    await expect(page).toHaveURL(/#\/tricky-store\/keybox$/)
  })

  test("reports a download the site could not serve", async ({ page, open }) => {
    await page.route("https://keybox.kowx712.cc/**", (route) =>
      route.fulfill({
        contentType: "text/html",
        body: site(`{ type: "error", error: "download_failed", identity: "Pixel-42" }`),
      }),
    )
    await open("/tricky-store/keybox/repo")
    await expect(page.getByText("Download failed for Pixel-42")).toBeVisible()
  })
})

test.describe("backends", () => {
  test("OhMyKeymint has no per-app modes but a boolean policy", async ({ page, open }) => {
    await open("/tricky-store", { query: { backend: "oh-my-keymint" } })
    await expect(page.getByText("OMK", { exact: true })).toBeVisible()
    await expect(page.getByRole("button", { name: /Mode and policy of/ })).toHaveCount(0)
    await open("/tricky-store/policy", { query: { backend: "oh-my-keymint" } })
    await expect(page.getByRole("switch", { name: "Bootloader locked" })).toBeVisible()
  })

  test("says when the module in use is disabled", async ({ page, open }) => {
    await open("/tricky-store", { query: { disabled: "1" } })
    await expect(page.getByText(/is disabled in your root manager/)).toBeVisible()
    await expect(page.locator("main").getByText("Tricky Store", { exact: true })).toHaveCount(1)
  })

  test("explains what to install when no keystore module exists", async ({ page, open }) => {
    await open("/tricky-store", { query: { backend: "none" } })
    await expect(page.getByText("No keystore module found")).toBeVisible()
  })

  test("opens straight into the manager from a keystore module's WebUI", async ({ page, open }) => {
    await open("/", { query: { host: "tricky_store" } })
    await expect(page).toHaveURL(/#\/tricky-store$/)
  })
})

test("saves prop settings", async ({ page, open }) => {
  await open("/tricky-store/props")
  await page.getByLabel("Boot Hash").fill("zz")
  await expect(page.getByText("Enter 64 hexadecimal characters")).toBeVisible()
  await page.getByLabel("Boot Hash").fill("AB".repeat(32))
  await page.getByRole("button", { name: "Save" }).click()
  await expect(page.getByText("Verified Boot Hash saved successfully")).toBeVisible()
})
