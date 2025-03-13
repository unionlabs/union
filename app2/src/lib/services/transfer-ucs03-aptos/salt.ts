import { toHex } from "viem"
import { Effect } from "effect"

/**
 * Effect that generates cryptographically secure random salts
 * used to prevent transfer hash collisions
 */
export const generateSalt = Effect.sync(() => {
  const saltHex = new Uint8Array(14)
  return toHex(saltHex)
})
