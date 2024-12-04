import { raise } from "./utilities/index.ts"
import { bech32, hex, bytes } from "@scure/base"
import type { Bech32Address, HexAddress } from "./types.ts"
import { isValidBech32Address } from "./utilities/address.ts"

/**
 * convert a bech32 address (cosmos, osmosis, union addresses) to hex address (evm)
 * @example
 * ```ts
 * bech32AddressToHex({ address: "union1qp0wtsfltjk9rnvyu3fkdv0s0skp4y5y3py96f" })
 * ```
 */
export function bech32AddressToHex({ address }: { address: string }): HexAddress {
  if (!isValidBech32Address(address)) raise(`Invalid bech32 address: ${address}`)
  const { bytes } = bech32.decodeToBytes(address)
  return `0x${bytesToHex(bytes)}`
}

/**
 * convert an Hex address (evm) to a bech32 address (cosmos, osmosis, union addresses)
 * @example
 * ```ts
 * hexAddressToBech32({
 *   bech32Prefix: "union",
 *   address: "0x779877A7B0D9E8603169DdbD7836e478b4624789"
 * })
 * ```
 */
export function hexAddressToBech32({
  address,
  bech32Prefix
}: { address: HexAddress; bech32Prefix: string }): Bech32Address {
  const words = bech32.toWords(hexToBytes(address))
  return bech32.encode(bech32Prefix, words, false)
}

/**
 * convert a bech32 address (cosmos, osmosis, union addresses) to a bech32 address with a different prefix
 * @example
 * ```ts
 * bech32ToBech32Address({
 *   toPrefix: "stride",
 *   address: "union1qp0wtsfltjk9rnvyu3fkdv0s0skp4y5y3py96f",
 * })
 * ```
 */
export function bech32ToBech32Address<ToPrefix extends string>({
  address,
  toPrefix
}: { address: string; toPrefix: ToPrefix }): Bech32Address<ToPrefix> {
  if (!isValidBech32Address(address)) raise(`Invalid bech32 address: ${address}`)
  return bech32.encode(toPrefix, bech32.decode(address).words, false) as Bech32Address<ToPrefix>
}

/**
 * convert a byte array to a bech32 address with a different prefix
 * @example
 * ```ts
 * bytesToBech32Address({
 *   toPrefix: "stride",
 *   bytes: new Uint8Array([1, 2, 3]),
 * })
 * ```
 */
export function bytesToBech32Address<ToPrefix extends string>({
  bytes,
  toPrefix
}: { bytes: Uint8Array; toPrefix: ToPrefix }): Bech32Address<ToPrefix> {
  return bech32.encode(toPrefix, bytes, false) as Bech32Address<ToPrefix>
}

export function bech32ToBytes(bech32Address: string): Uint8Array {
  const { words } = bech32.decode(bech32Address as `${string}1${string}`)
  return bech32.fromWords(words)
}

/**
 * convert a byte array to a hex string
 * @example
 * ```ts
 * bytesToHex(new Uint8Array([1, 2, 3]))
 * ```
 */
export function bytesToHex(bytes: Uint8Array): string {
  return hex.encode(bytes)
}

/**
 * convert a hex string to a byte array
 * @example
 * ```ts
 * hexToBytes("0x779877A7B0D9E8603169DdbD7836e478b4624789")
 * ```
 */
export function hexToBytes(hexString: string): Uint8Array {
  return bytes("hex", hexString.indexOf("0x") === 0 ? hexString.slice(2) : hexString)
}
