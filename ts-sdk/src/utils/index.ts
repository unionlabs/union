import { toHex, type Hex } from "viem"

export { extractErrorDetails } from "./extract-error-details.js"

export const generateSalt = () => {
  const rawSalt = new Uint8Array(32)
  for (let i = 0; i < rawSalt.length; i++) {
    rawSalt[i] = Math.floor(Math.random() * 256)
  }
  return toHex(rawSalt) as Hex
}

export const generateSaltAptos = () => {
  const rawSalt = new Uint8Array(14)
  for (let i = 0; i < rawSalt.length; i++) {
    rawSalt[i] = Math.floor(Math.random() * 256)
  }
  return toHex(rawSalt) as Hex
}
