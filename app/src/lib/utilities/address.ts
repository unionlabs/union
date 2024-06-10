import { bech32 } from "bech32"

export const rawToHex = (raw: Uint8Array): string =>
  `${Array.from(raw)
    .map(i => i.toString(16).padStart(2, "0"))
    .join("")
    .toLowerCase()}`

export const rawToBech32 = (prefix: string, raw: Uint8Array): string => {
  const words = bech32.toWords(raw)
  return bech32.encode(prefix, words)
}
