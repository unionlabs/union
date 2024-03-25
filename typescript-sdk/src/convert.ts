import { bech32 } from "@scure/base"
import { raise } from "./utilities.ts"
import { fromBech32 } from "@cosmjs/encoding"
import type { UnionAddress, EvmAddress } from "./types.ts"
import { getAddress, isHex, isAddress } from "viem"

export const isValidEvmAddress = (address: unknown): address is EvmAddress =>
  typeof address === "string" && //
  isAddress(address) &&
  getAddress(address) === address

export function isValidCosmosAddress(
  address: unknown,
  { expectedPrefixes }: { expectedPrefixes: ["union"] } = { expectedPrefixes: ["union"] }
): address is UnionAddress {
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
  if (hexString.length % 2 !== 0) throw new Error("Hex must have an even number of characters")

  const arrayBuffer = new Uint8Array(hexString.length / 2)
  for (let index = 0; index < hexString.length; index += 2) {
    arrayBuffer[index / 2] = Number.parseInt(hexString.substring(index, index + 2), 16)
  }
  return arrayBuffer
}

export const truncateUnionAddress = (address: string, length = 6) =>
  length > 0 ? `${address.slice(0, length)}...${address.slice(-length)}` : address

export const truncateEvmAddress = (address: string, length = 6) =>
  length > 0 ? `${address.slice(0, length)}...${address.slice(-length)}` : address

export const convertByteArrayToHex = (byteArray: Uint8Array): string =>
  byteArray.reduce((hex, byte) => hex + byte.toString(16).padStart(2, "0"), "").toUpperCase()

export function unionToEvmAddress(address: string): EvmAddress {
  if (!isValidCosmosAddress(address)) raise("Invalid Cosmos address")
  const { words } = bech32.decode(address)
  return getAddress(`0x${Buffer.from(bech32.fromWords(words)).toString("hex")}`)
}

export const evmToCosmosAddress = (address: EvmAddress): UnionAddress => {
  const words = bech32.toWords(Buffer.from(address.slice(2), "hex"))
  return bech32.encode("union", words)
}

export const normalizeToCosmosAddress = (address: string): UnionAddress =>
  isHex(address) ? evmToCosmosAddress(address) : (address as UnionAddress)

export const normalizeToEvmAddress = (address: string): EvmAddress =>
  isHex(address) ? address : unionToEvmAddress(address as UnionAddress)

export const munoToUno = (muno: string | number) => (Number(muno) / 1e6).toFixed(6)
