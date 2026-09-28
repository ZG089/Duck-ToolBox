import { usePreferredDark, useStorage } from "@vueuse/core"
import { computed, ref, watchEffect } from "vue"

export type ThemePreference = "system" | "light" | "dark"

const preference = useStorage<ThemePreference>("duck-toolbox/theme", "system")
const managerDark = ref<boolean | null>(null)

/**
 * Dark or light, judged from the KernelSU manager's Monet background (`--background` from
 * internal/colors.css), or `null` when the manager does not share its palette.
 */
function readManagerTheme(): boolean | null {
  const value = getComputedStyle(document.documentElement).getPropertyValue("--background").trim()
  const match = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})/i.exec(value)
  if (!match) return null
  const [red, green, blue] = match.slice(1).map((hex) => Number.parseInt(hex, 16) / 255)
  return 0.2126 * red! + 0.7152 * green! + 0.0722 * blue! < 0.5
}

/** Applies the `dark` class for the whole session, following the preference. */
export function startTheme(): void {
  const systemDark = usePreferredDark()
  managerDark.value = readManagerTheme()
  document
    .querySelector<HTMLLinkElement>('link[href="/internal/colors.css"]')
    ?.addEventListener("load", () => (managerDark.value = readManagerTheme()))

  const dark = computed(() =>
    preference.value === "system"
      ? (managerDark.value ?? systemDark.value)
      : preference.value === "dark",
  )
  watchEffect(() => document.documentElement.classList.toggle("dark", dark.value))
}

export function useThemePreference() {
  return preference
}
