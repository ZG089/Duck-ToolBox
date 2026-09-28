import { useI18n } from "vue-i18n"

import type { DotPath } from "./paths"

type Params = Record<string, unknown> | unknown[]

/**
 * Translation scoped to one namespace, typed against that namespace's English messages:
 * `useFeatureI18n<Messages>("trickyStore").t("menu.keybox")`.
 */
export function useFeatureI18n<Messages>(namespace: string) {
  const { t, te, locale } = useI18n()
  return {
    locale,
    t: (key: DotPath<Messages>, params?: Params) =>
      params === undefined ? t(`${namespace}.${key}`) : t(`${namespace}.${key}`, params as never),
    has: (key: string) => te(`${namespace}.${key}`),
    /** For keys built at runtime (schema field labels, error codes, ...). */
    dynamic: (key: string, fallback: string) =>
      te(`${namespace}.${key}`) || te(`${namespace}.${key}`, "en")
        ? t(`${namespace}.${key}`)
        : fallback,
  }
}
