/**
 * Utility functions for formatting chain parameters
 */

/**
 * Format a duration string (e.g., "1209600s") to human readable
 */
export function formatDuration(duration: string): string {
  // Parse seconds from duration string like "1209600s" or nanoseconds
  let seconds: number

  if (duration.endsWith("s")) {
    seconds = parseInt(duration.slice(0, -1))
  } else if (duration.endsWith("ns")) {
    seconds = parseInt(duration.slice(0, -2)) / 1e9
  } else {
    seconds = parseInt(duration)
  }

  if (isNaN(seconds)) return duration

  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)

  const parts: string[] = []
  if (days > 0) parts.push(`${days}d`)
  if (hours > 0) parts.push(`${hours}h`)
  if (minutes > 0 && days === 0) parts.push(`${minutes}m`)

  return parts.length > 0 ? parts.join(" ") : `${seconds}s`
}

/**
 * Format a decimal string (e.g., "0.050000000000000000") to percentage for param display
 */
export function formatParamPercent(decimal: string): string {
  const num = parseFloat(decimal)
  if (isNaN(num)) return decimal

  // Handle very small numbers
  if (num < 0.0001 && num > 0) {
    return `${(num * 100).toExponential(2)}%`
  }

  return `${(num * 100).toFixed(2)}%`
}

/**
 * Format a large number with commas for param display
 */
export function formatParamNumber(num: string | number): string {
  const n = typeof num === "string" ? parseInt(num) : num
  if (isNaN(n)) return String(num)
  return n.toLocaleString()
}

/**
 * Format boolean to Yes/No
 */
export function formatBool(value: boolean): string {
  return value ? "Yes" : "No"
}

/**
 * Clean up param key for display (e.g., "min_signed_per_window" -> "Min Signed Per Window")
 */
export function formatParamKey(key: string): string {
  return key
    .split("_")
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(" ")
}
