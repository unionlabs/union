export const isBrowser = typeof document !== "undefined"

export function raise(error: unknown): never {
  throw typeof error === "string" ? new Error(error) : error
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

export const elementHasFocus = (element: Element | null): element is HTMLElement =>
  element === document.activeElement

export async function fetcher<T>(url: string, options?: RequestInit) {
  const response = await fetch(url, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
      ...options?.headers
    }
  })
  if (!response.ok) {
    raise(
      `\n ${response.status} - Failed to fetch from ${url}:\n ${
        response.statusText
      }\n ${await response.text()}\n`
    )
  }
  const data = (await response.json()) as T
  return data
}

export async function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

export const generateRandomInteger = (min: number, max: number) =>
  Math.floor(Math.random() * (max - min + 1)) + min

export const repeatArray = <T>(array: Array<T>, times: number): Array<T> =>
  Array.from({ length: times }).flatMap(() => array)

export function debounce<T extends (...args: Array<any>) => void>(
  handler: T,
  delay = 500
): (...args: Parameters<T>) => void {
  let id: number
  return (...args: Parameters<T>) => {
    window.clearTimeout(id)
    id = window.setTimeout(handler, delay, ...args)
  }
}

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
