import { bech32 } from "@scure/base"
import type { HexAddress, Bech32Address } from "../types.ts"

/**
 * check if a string is a valid cosmos transaction hash
 * @example
 * ```ts
 * isValidCosmosTxHash("A6E276CE66CDB35C0CAAC49EC9AAB3CB2CF8A34C807A4C729EA385E64C88D69B")
 * ```
 */
export function isValidCosmosTxHash(hash: unknown): hash is string {
  if (typeof hash !== "string") return false
  return typeof hash === "string" && /^[A-Fa-f0-9]{64}$/.test(hash)
}

/**
 * check if a string is a valid evm transaction hash
 * @example
 * ```ts
 * isValidEvmTxHash("0xA6E276CE66CDB35C0CAAC49EC9AAB3CB2CF8A34C807A4C729EA385E64C88D69B")
 * ```
 */
export function isValidEvmTxHash(hash: unknown): hash is string {
  if (typeof hash !== "string" || hash.indexOf("0x") !== 0) return false
  return typeof hash === "string" && /^0x([A-Fa-f0-9]{64})$/.test(hash)
}

/**
 * check if a string is a valid evm address
 * @example
 * ```ts
 * isValidEvmAddress("0xA6E276CE66CDB35C0CAAC49EC9AAB3CB2CF8A34C")
 * ```
 */
export const isValidEvmAddress = (address: unknown): address is HexAddress =>
  typeof address === "string" && /^0x[a-fA-F0-9]{40}$/.test(address)

/**
 * check if a string is a valid bech32 address
 * @example
 * ```ts
 * isValidBech32Address("union1qp0wtsfltjk9rnvyu3fkdv0s0skp4y5y3py96f")
 * ```
 */
export function isValidBech32Address(address: unknown): address is Bech32Address {
  if (typeof address !== "string") return false

  try {
    const { prefix: _, words } = bech32.decode(address)
    const size = words.length
    if ([20, 32].indexOf(size) === -1) return false

    return true
  } catch {
    return false
  }
}

/**
 * truncate an address to a given length
 * @example
 * ```ts
 * truncateAddress({
 *   length: 6,
 *   address: "union1qp0wtsfltjk9rnvyu3fkdv0s0skp4y5y3py96f",
 * })
 * ```
 */
export const truncateAddress = ({
  address,
  length = 6
}: {
  address: string
  length?: number
}): string => (length > 0 ? `${address.slice(0, length)}...${address.slice(-length)}` : address)
