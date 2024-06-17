import { isAddress, getAddress } from "viem"
import { fromBech32 } from "@cosmjs/encoding"
import type { HexAddress, Bech32Address } from "../types.ts"

export const truncateBech32Address = (address: string, length = 6) =>
  length > 0 ? `${address.slice(0, length)}...${address.slice(-length)}` : address

export const truncateEvmAddress = (address: string, length = 6) =>
  length > 0 ? `${address.slice(0, length)}...${address.slice(-length)}` : address

export function truncateAddress({
  address,
  type,
  length = 6
}: {
  address: string
  type: "bech32" | "hex"
  length?: number
}) {
  return type === "bech32"
    ? truncateBech32Address(address, length)
    : truncateEvmAddress(address, length)
}

export const isValidEvmAddress = (address: unknown): address is HexAddress =>
  typeof address === "string" && isAddress(address) && getAddress(address) === address

export function isValidBech32Address(
  address: unknown,
  { expectedPrefixes }: { expectedPrefixes: ["union"] } = { expectedPrefixes: ["union"] }
): address is Bech32Address {
  if (typeof address !== "string") return false

  try {
    const { prefix, data } = fromBech32(address)
    if (expectedPrefixes && !expectedPrefixes.includes(prefix)) return false

    const size = data.length
    if ([20, 32].indexOf(size) === -1) return false

    return true
  } catch (error) {
    return false
  }
}
