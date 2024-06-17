import { bech32 } from "@scure/base"
import { getAddress, isHex } from "viem"
import { raise } from "./utilities/index.ts"
import type { Bech32Address, HexAddress } from "./types.ts"
import { isValidBech32Address } from "./utilities/address.ts"

/**
 * convert a bech32 address (cosmos, osmosis, union addresses) to hex address (evm)
 */
export function bech32AddressToHex({ address }: { address: string }): HexAddress {
  if (!isValidBech32Address(address)) raise("Invalid Cosmos address")
  const { words } = bech32.decode(address)
  return getAddress(`0x${Buffer.from(bech32.fromWords(words)).toString("hex")}`)
}

/**
 * convert an Hex address (evm) to a bech32 address (cosmos, osmosis, union addresses)
 */
export function hexAddressToBech32({
  address,
  bech32Prefix
}: { address: HexAddress; bech32Prefix: string }): Bech32Address {
  const words = bech32.toWords(Buffer.from(address.slice(2), "hex"))
  return bech32.encode(bech32Prefix, words)
}

/**
 * @credit https://stackoverflow.com/a/78013306/10605502
 */
const LUT_HEX_4b = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F"]
const LUT_HEX_8b = new Array(0x100) satisfies Array<string>
for (let index = 0; index < 0x100; index++) {
  LUT_HEX_8b[index] = `${LUT_HEX_4b[(index >>> 4) & 0xf]}${LUT_HEX_4b[index & 0xf]}`
}
let out = ""
export function uint8ArrayToHexString(uintArray: Uint8Array): string {
  for (let index = 0, edx = uintArray.length; index < edx; index++) {
    out += LUT_HEX_8b[uintArray[index] as number]
  }
  return out
}

export function hexStringToUint8Array(hexString: string) {
  if (hexString.length % 2 !== 0) raise("Hex must have an even number of characters")

  const arrayBuffer = new Uint8Array(hexString.length / 2)
  for (let index = 0; index < hexString.length; index += 2) {
    arrayBuffer[index / 2] = Number.parseInt(hexString.substring(index, index + 2), 16)
  }
  return arrayBuffer
}

export const convertByteArrayToHex = (byteArray: Uint8Array): string =>
  byteArray.reduce((hex, byte) => hex + byte.toString(16).padStart(2, "0"), "").toUpperCase()

export const normalizeToCosmosAddress = (address: string): Bech32Address =>
  isHex(address)
    ? hexAddressToBech32({ address, bech32Prefix: "union" })
    : (address as Bech32Address)

export const normalizeToEvmAddress = (address: string): HexAddress =>
  isHex(address) ? address : bech32AddressToHex({ address })

export const munoToUno = (muno: string | number) => (Number(muno) / 1e6).toFixed(6)
