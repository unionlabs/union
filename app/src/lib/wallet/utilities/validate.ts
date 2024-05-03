import { bech32 } from "@scure/base"
import { isAddress, getAddress } from "viem"
import type { CosmosAddress, EvmAddress } from "$lib/wallet/types.ts"

export function isValidEvmAddress(address: unknown): address is EvmAddress {
  return (
    typeof address === "string" && //
    isAddress(address) &&
    getAddress(address) === address
  )
}

export function isValidCosmosAddress(
  address: unknown,
  expectedPrefixes = ["union"]
): address is CosmosAddress {
  if (typeof address !== "string") return false

  try {
    const { prefix, words } = bech32.decode(address)
    if (expectedPrefixes && !expectedPrefixes.includes(prefix)) return false

    const size = bech32.fromWords(words).length
    if (size !== 20 && size !== 32) return false

    return true
  } catch (error) {
    return false
  }
}
