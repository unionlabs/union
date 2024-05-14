import { isAddress, getAddress } from "viem"
import { normalizeBech32 } from "@cosmjs/encoding"
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
    const nromalized = normalizeBech32(address)
    return expectedPrefixes.includes(nromalized.slice(0, nromalized.indexOf("1")))
  } catch (error) {
    return false
  }
}
