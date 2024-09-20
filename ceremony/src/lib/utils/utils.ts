import { browser } from "$app/environment"

export function getNumberSuffix(n: number | null): string {
  if (n == null) return ""

  const lastDigit = n % 10
  const lastTwoDigits = n % 100

  switch (lastDigit) {
    case 1:
      if (lastTwoDigits !== 11) return "st"
      break
    case 2:
      if (lastTwoDigits !== 12) return "nd"
      break
    case 3:
      if (lastTwoDigits !== 13) return "rd"
      break
    default:
      break
  }

  return "th"
}

export function isSafari(): boolean {
  if (!browser) {
    return false
  }

  const ua = navigator.userAgent.toLowerCase()
  return ua.indexOf("safari") > -1 && ua.indexOf("chrome") === -1
}
