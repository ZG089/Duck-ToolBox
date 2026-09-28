import type { Page } from "@playwright/test"
import { test as base, expect } from "@playwright/test"

interface Options {
  locale?: string
  theme?: "light" | "dark"
  /** Extra query parameters for the mock host, e.g. `{ backend: "oh-my-keymint" }`. */
  query?: Record<string, string>
}

async function open(page: Page, route: string, options: Options = {}) {
  await page.addInitScript(({ locale, theme }) => {
    if (locale) localStorage.setItem("duck-toolbox/locale", locale)
    if (theme) localStorage.setItem("duck-toolbox/theme", theme)
  }, options)
  const query = new URLSearchParams({ latency: "20", ...options.query })
  await page.goto(`./?${query}#${route}`)
}

export const test = base.extend<{ open: (route: string, options?: Options) => Promise<void> }>({
  open: async ({ page }, use) => {
    await use((route, options) => open(page, route, options))
  },
})

export { expect }
