import { isHex } from "viem"
import { bech32 } from "@scure/base"
import { raise } from "./utilities/index.ts"
import type { Bech32Address, HexAddress } from "./types.ts"

export const convertByteArrayToHex = (byteArray: Uint8Array): string =>
  byteArray.reduce((hex, byte) => hex + byte.toString(16).padStart(2, "0"), "")

/**
 * convert a bech32 address (cosmos, osmosis, union addresses) to hex address (evm)
 */
export function bech32AddressToHex({ address }: { address: string }): HexAddress {
  const { words } = bech32.decode(address)
  const byteArray = bech32.fromWords(words)
  return `0x${convertByteArrayToHex(byteArray)}`
}

/**
 * convert an Hex address (evm) to a bech32 address (cosmos, osmosis, union addresses)
 */
export function hexAddressToBech32({
  address,
  bech32Prefix
}: { address: HexAddress; bech32Prefix: string }): Bech32Address {
  if (!isHex(address)) raise("Invalid hex address")
  const words = bech32.toWords(hexStringToUint8Array(address.slice(2)))
  return bech32.encode(bech32Prefix, words)
}

export function bech32ToBech32Address<ToPrefix extends string>({
  address,
  toPrefix
}: { address: string; toPrefix: ToPrefix }): Bech32Address<ToPrefix> {
  return bech32.encode(toPrefix, bech32.decode(address).words) as Bech32Address<ToPrefix>
}

export function bytesToBech32Address<ToPrefix extends string>({
  bytes,
  toPrefix
}: { bytes: Uint8Array; toPrefix: ToPrefix }): Bech32Address<ToPrefix> {
  return bech32.encode(toPrefix, bech32.toWords(bytes)) as Bech32Address<ToPrefix>
}

/**
 * @credit https://stackoverflow.com/a/78013306/10605502
 */
const LUT_HEX_4b = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F"]
const LUT_HEX_8b = new Array(0x100) as Array<string>
for (let n = 0; n < 0x100; n++) {
  LUT_HEX_8b[n] = `${LUT_HEX_4b[(n >>> 4) & 0xf]}${LUT_HEX_4b[n & 0xf]}`
}
export function uint8ArrayToHexString(uintArray: Uint8Array): string {
  let out = ""
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

export const normalizeToCosmosAddress = ({
  address,
  bech32Prefix
}: { address: string; bech32Prefix: string }): Bech32Address =>
  isHex(address) ? hexAddressToBech32({ address, bech32Prefix }) : (address as Bech32Address)

export const normalizeToEvmAddress = (address: string): HexAddress =>
  isHex(address) ? address : bech32AddressToHex({ address })

export const munoToUno = (muno: string | number) => (Number(muno) / 1e6).toFixed(6)
