import { expect, test } from "./fixtures"

// Full-page captures for review; CI uploads them as an artifact.
const pages = [
  { name: "home", route: "/", ready: "Tricky Store" },
  { name: "tricky-store", route: "/tricky-store", ready: "Key Attestation" },
  { name: "keybox", route: "/tricky-store/keybox", ready: "AOSP" },
  { name: "policy", route: "/tricky-store/policy", ready: "System patch" },
  { name: "rkp", route: "/rkp", ready: "Device" },
  { name: "system", route: "/system", ready: "Duck ToolBox" },
  { name: "settings", route: "/settings", ready: "Language" },
]

const variants = [
  { name: "light", theme: "light", locale: "en" },
  { name: "dark", theme: "dark", locale: "en" },
  { name: "rtl", theme: "light", locale: "ar" },
  { name: "zh", theme: "light", locale: "zh-CN" },
] as const

for (const variant of variants) {
  test.describe(variant.name, () => {
    for (const target of pages) {
      test(target.name, async ({ page, open }) => {
        await open(target.route, { theme: variant.theme, locale: variant.locale })
        if (variant.locale === "en")
          await expect(page.getByText(target.ready).first()).toBeVisible()
        await page.waitForLoadState("networkidle")
        await page.screenshot({
          path: `test-results/screenshots/${variant.name}-${target.name}.png`,
          fullPage: true,
        })
      })
    }
  })
}
