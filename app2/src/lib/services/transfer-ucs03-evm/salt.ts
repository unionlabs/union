import { type Hex, toHex } from "viem"
import { Effect } from "effect"

/**
 * Effect that generates cryptographically secure random salts
 * used to prevent transfer hash collisions
 */
export const generateSalt = Effect.sync(() => {
  const rawSalt = new Uint8Array(32)
  crypto.getRandomValues(rawSalt)
  return toHex(rawSalt) as Hex
})
