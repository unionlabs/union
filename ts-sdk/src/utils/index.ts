import { toHex, type Hex } from "viem"

export { extractErrorDetails } from "./extract-error-details.js"

export const generateSalt = () => {
  const rawSalt = new Uint8Array(32)
  // TODO: fix salt!
  crypto.getRandomValues(rawSalt)
  return toHex(rawSalt) as Hex
}
