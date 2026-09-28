import { usePreferredDark, useStorage } from "@vueuse/core"
import { readonly, ref, watchEffect, type Ref } from "vue"

export type ThemePreference = "system" | "light" | "dark"

const preference = useStorage<ThemePreference>("duck-toolbox/theme", "system")
const dark = ref(false)

/** Applies the `dark` class for the whole session, following the preference. */
export function startTheme(): void {
  const systemDark = usePreferredDark()
  watchEffect(() => {
    dark.value = preference.value === "system" ? systemDark.value : preference.value === "dark"
    document.documentElement.classList.toggle("dark", dark.value)
  })
}

export function useThemePreference() {
  return preference
}

/** Whether the dark theme is showing, for components that theme themselves. */
export function useDarkTheme(): Readonly<Ref<boolean>> {
  return readonly(dark)
}
