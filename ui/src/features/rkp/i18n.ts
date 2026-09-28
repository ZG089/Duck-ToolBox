import { useFeatureI18n } from "@/core/i18n"

import type messages from "./locales/en.json"

export const NAMESPACE = "rkp"

export function useRkpI18n() {
  return useFeatureI18n<typeof messages>(NAMESPACE)
}
