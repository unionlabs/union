import { fromBech32 } from "@cosmjs/encoding"
import type { HexAddress, Bech32Address } from "../types.ts"

export function isValidCosmosTxHash(hash: unknown): hash is string {
  if (typeof hash !== "string") return false
  return typeof hash === "string" && /^[A-Fa-f0-9]{64}$/.test(hash)
}

export function isValidEvmTxHash(hash: unknown): hash is string {
  if (typeof hash !== "string" || hash.indexOf("0x") !== 0) return false
  return typeof hash === "string" && /^0x([A-Fa-f0-9]{64})$/.test(hash)
}

export const isValidEvmAddress = (address: unknown): address is HexAddress =>
  typeof address === "string" && /^0x[a-fA-F0-9]{40}$/.test(address)

export function isValidBech32Address(address: unknown): address is Bech32Address {
  if (typeof address !== "string") return false

  try {
    const { prefix: _, data } = fromBech32(address)
    const size = data.length
    if ([20, 32].indexOf(size) === -1) return false

    return true
  } catch {
    return false
  }
}

export const truncateAddress = ({
  address,
  length = 6
}: {
  address: string
  length?: number
}) => (length > 0 ? `${address.slice(0, length)}...${address.slice(-length)}` : address)
