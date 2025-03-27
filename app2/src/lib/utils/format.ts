import { bech32 } from "@scure/base"
type Bech32Address<T extends string = string> = `${T}1${string}`;
/**
 * Truncates a string based on the given parameters
 * @param {string} str - The string to truncate
 * @param {number} showChars - Number of characters to show on each end when truncated
 * @param {string} position - Where to place the ellipsis: 'start', 'middle', or leave empty for end
 * @returns {string} - The truncated string
 */
export function truncate(str: string, showChars: number, position: string): string {
  // If string is shorter than or equal to the total characters to show, return it as is
  if (str.length <= showChars * 2 || showChars <= 0) {
    return str
  }

  const ellipsis = "…"

  switch (position.toLowerCase()) {
    case "start":
      // Show ellipsis at the start
      return ellipsis + str.slice(str.length - showChars)

    case "middle": {
      // Show ellipsis in the middle
      const firstPart = str.slice(0, showChars)
      const lastPart = str.slice(str.length - showChars)
      return firstPart + ellipsis + lastPart
    }
    default:
      // Show ellipsis at the end (default)
      return str.slice(0, showChars) + ellipsis
  }
}


// TODO: This needs to be tested
/**
 * check if a string is a valid bech32 address
 */
export function isValidBech32Address(address: unknown): address is Bech32Address {
  if (typeof address !== "string") return false
  
  try {
    bech32.decode(address as Bech32Address)
    return true
  } catch {
    return false
  }
}
