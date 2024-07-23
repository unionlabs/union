import { isHex, bytesToHex } from "viem"
import { hexToUint8Array } from "uint8array-extras"
import { fromBech32, toBech32 } from "@cosmjs/encoding"

import type { CosmosAddress, EvmAddress } from "$lib/wallet/types.ts"

export function cosmosToEvmAddress(address: CosmosAddress): EvmAddress {
  return bytesToHex(fromBech32(address).data)
}

export function evmToCosmosAddress(address: EvmAddress): CosmosAddress {
  const addressToBuffer = hexToUint8Array(address)
  return toBech32("union", addressToBuffer) as CosmosAddress
}

export const normalizeToCosmosAddress = (address: string): CosmosAddress =>
  isHex(address) ? evmToCosmosAddress(address) : (address as CosmosAddress)

export const normalizeToEvmAddress = (address: string): EvmAddress =>
  isHex(address) ? address : cosmosToEvmAddress(address as CosmosAddress)
