import { bech32 } from "@scure/base"
import { getAddress, isHex } from "viem"
import { uint8ArrayToHex, hexToUint8Array } from "uint8array-extras"
import type { CosmosAddress, EvmAddress } from "$lib/wallet/types.ts"
import { isValidCosmosAddress } from "$lib/wallet/utilities/validate.ts"

export function cosmosToEvmAddress(address: CosmosAddress): EvmAddress {
  if (!isValidCosmosAddress(address)) throw new Error("Invalid Cosmos address")
  const { words } = bech32.decode(address)
  return getAddress(`0x${uint8ArrayToHex(bech32.fromWords(words))}`)
}

export function evmToCosmosAddress(address: EvmAddress): CosmosAddress {
  const words = bech32.toWords(hexToUint8Array(address.slice(2)))
  return bech32.encode("union", words)
}

export const normalizeToCosmosAddress = (address: string): CosmosAddress =>
  isHex(address) ? evmToCosmosAddress(address) : (address as CosmosAddress)

export const normalizeToEvmAddress = (address: string): EvmAddress =>
  isHex(address) ? address : cosmosToEvmAddress(address as CosmosAddress)
