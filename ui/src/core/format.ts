import { i18n } from "@/core/i18n"

function locale(): string {
  return i18n.global.locale.value
}

export function formatBytes(bytes: number): string {
  const units = ["byte", "kilobyte", "megabyte", "gigabyte"] as const
  let value = bytes
  let unit = 0
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024
    unit += 1
  }
  return new Intl.NumberFormat(locale(), {
    style: "unit",
    unit: units[unit],
    unitDisplay: "short",
    maximumFractionDigits: unit === 0 ? 0 : 1,
  }).format(value)
}

export function formatDateTime(unixSeconds: number): string {
  return new Intl.DateTimeFormat(locale(), { dateStyle: "medium", timeStyle: "short" }).format(
    unixSeconds * 1000,
  )
}

/** "3 minutes ago", "yesterday", ... */
export function formatRelative(unixSeconds: number, now = Date.now()): string {
  const seconds = Math.round(unixSeconds - now / 1000)
  const steps: [Intl.RelativeTimeFormatUnit, number][] = [
    ["second", 60],
    ["minute", 60],
    ["hour", 24],
    ["day", 30],
    ["month", 12],
    ["year", Number.POSITIVE_INFINITY],
  ]
  let value = seconds
  for (const [unit, size] of steps) {
    if (Math.abs(value) < size) {
      return new Intl.RelativeTimeFormat(locale(), { numeric: "auto" }).format(value, unit)
    }
    value = Math.round(value / size)
  }
  return formatDateTime(unixSeconds)
}
