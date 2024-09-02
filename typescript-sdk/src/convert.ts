import { isHex } from "viem"
import { bech32 } from "@scure/base"
import { raise } from "./utilities/index.ts"
import type { Bech32Address, HexAddress } from "./types.ts"

/**
 * convert a byte array to a hex string
 * @example
 * ```ts
 * bytesToHex(new Uint8Array([1, 2, 3]))
 * ```
 */
export const bytesToHex = (byteArray: Uint8Array): string =>
  byteArray.reduce((hex, byte) => hex + byte.toString(16).padStart(2, "0"), "")

/**
 * convert a bech32 address (cosmos, osmosis, union addresses) to hex address (evm)
 * @example
 * ```ts
 * bech32AddressToHex({ address: "union1qp0wtsfltjk9rnvyu3fkdv0s0skp4y5y3py96f" })
 * ```
 */
export function bech32AddressToHex({ address }: { address: string }): HexAddress {
  const { words } = bech32.decode(address)
  const byteArray = bech32.fromWords(words)
  return `0x${bytesToHex(byteArray)}`
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
  if (!isHex(address)) raise("Invalid hex address")
  const words = bech32.toWords(hexStringToUint8Array(address.slice(2)))
  return bech32.encode(bech32Prefix, words)
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
  return bech32.encode(toPrefix, bech32.decode(address).words) as Bech32Address<ToPrefix>
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
  return bech32.encode(toPrefix, bech32.toWords(bytes)) as Bech32Address<ToPrefix>
}

/**
 * @credit https://stackoverflow.com/a/78013306/10605502
 */
const LUT_HEX_4b: Array<string> = [
  "0",
  "1",
  "2",
  "3",
  "4",
  "5",
  "6",
  "7",
  "8",
  "9",
  "A",
  "B",
  "C",
  "D",
  "E",
  "F"
]
const LUT_HEX_8b: Array<string> = new Array(0x100)
for (let n = 0; n < 0x100; n++) {
  LUT_HEX_8b[n] = `${LUT_HEX_4b[(n >>> 4) & 0xf]}${LUT_HEX_4b[n & 0xf]}`
}

/**
 * convert a hex string to a byte array
 * @example
 * ```ts
 * hexStringToUint8Array("0x779877A7B0D9E8603169DdbD7836e478b4624789")
 * ```
 */
export function uint8ArrayToHexString(uintArray: Uint8Array): string {
  let out = ""
  for (let index = 0, edx = uintArray.length; index < edx; index++) {
    out += LUT_HEX_8b[uintArray[index] as number]
  }
  return out
}

/**
 * convert a hex string to a byte array
 * @example
 * ```ts
 * hexStringToUint8Array("0x779877A7B0D9E8603169DdbD7836e478b4624789")
 * ```
 */
export function hexStringToUint8Array(hexString: string): Uint8Array {
  if (hexString.length % 2 !== 0) raise("Hex must have an even number of characters")

  const arrayBuffer = new Uint8Array(hexString.length / 2)
  for (let index = 0; index < hexString.length; index += 2) {
    arrayBuffer[index / 2] = Number.parseInt(hexString.substring(index, index + 2), 16)
  }
  return arrayBuffer
}
