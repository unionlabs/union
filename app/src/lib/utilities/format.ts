export const dollarize = (amount: string | number) =>
  new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD"
  }).format(Number(amount))

export function relativeTime({
  timestamp,
  lang = navigator?.language || "en"
}: {
  timestamp: Date | number | string
  lang?: string
}): string {
  if (!timestamp) return ""
  const rtf = new Intl.RelativeTimeFormat(lang, { numeric: "auto" })
  const timeMilliseconds =
    typeof timestamp === "number"
      ? timestamp
      : typeof timestamp === "string"
        ? Date.parse(timestamp)
        : timestamp.getTime()

  const deltaSeconds = Math.round((timeMilliseconds - Date.now()) / 1000)

  const second = 1
  const minute = 60 * second
  const hour = 60 * minute
  const day = 24 * hour
  const [week, month, year] = [7 * day, 30 * day, 365 * day]

  const cutoffs = [
    { cutoff: minute, unit: "minute" },
    { cutoff: hour, unit: "hour" },
    { cutoff: day, unit: "day" },
    { cutoff: week, unit: "week" },
    { cutoff: month, unit: "month" },
    { cutoff: year, unit: "year" },
    { cutoff: Number.MAX_SAFE_INTEGER, unit: "year" }
  ] satisfies Array<{ cutoff: number; unit: Intl.RelativeTimeFormatUnit }>

  const units = [
    "second",
    "minute",
    "hour",
    "day",
    "week",
    "month",
    "year"
  ] satisfies Array<Intl.RelativeTimeFormatUnit>

  const unitIndex = cutoffs.findIndex(({ cutoff }) => cutoff > Math.abs(deltaSeconds))
  const divisor = unitIndex ? cutoffs[unitIndex - 1].cutoff : 1

  return rtf.format(Math.round(deltaSeconds / divisor), units[unitIndex])
}

export function urlSearchParams(
  params: Record<string, string | number | boolean | undefined | null>
) {
  return new URLSearchParams(
    JSON.parse(
      JSON.stringify({
        ...params
      })
    ) as Record<string, string>
  )
}

export function truncate(str: string, show: number): string {
  // Don't truncate short strings
  if (str.length === 0 || str.length < show * 2 + 2) return str

  // Extract the first `show` characters and the last `show` characters
  const firstPart: string = str.slice(0, show)
  const lastPart: string = str.slice(-show)

  // Return the truncated string with the ellipsis character in-between
  return `${firstPart}\u2026${lastPart}`
}
