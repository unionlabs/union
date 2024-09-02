import { isAddress, getAddress } from "viem"
import { normalizeBech32 } from "@cosmjs/encoding"
import type { CosmosAddress, EvmAddress } from "$lib/wallet/types.ts"

export function isValidEvmAddress(address: unknown): address is EvmAddress {
  return typeof address === "string" && isAddress(address) && getAddress(address) === address
}

export function isValidCosmosAddress(
  address: unknown,
  expectedPrefixes: string[] | string = ["union"]
): address is CosmosAddress {
  if (typeof address !== "string") return false

  try {
    const normalized = normalizeBech32(address)
    const prefix = normalized.slice(0, normalized.indexOf("1"))

    const prefixes = Array.isArray(expectedPrefixes) ? expectedPrefixes : [expectedPrefixes]

    return prefixes.includes(prefix)
  } catch (error) {
    return false
  }
}

