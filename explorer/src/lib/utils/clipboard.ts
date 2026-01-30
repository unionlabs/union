import { browser } from "$app/environment"

/**
 * Copy text to clipboard with optional callback
 * @param text - Text to copy
 * @param onSuccess - Optional callback when copy succeeds
 */
export async function copyToClipboard(text: string, onSuccess?: () => void): Promise<boolean> {
  if (!browser) return false

  try {
    await navigator.clipboard.writeText(text)
    onSuccess?.()
    return true
  } catch (err) {
    console.error("Failed to copy to clipboard:", err)
    return false
  }
}
