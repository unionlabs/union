import { bytesToHex, type Address } from "viem"
import { fromBech32, toBech32 } from "@cosmjs/encoding"

/**
 * Get union address from ethereum address
 */
export function evmDecodeUnionAddress(ethereumAddress: Address) {
  const addressToBuffer = Buffer.from(ethereumAddress.slice(2), "hex")
  return toBech32("union", addressToBuffer)
}

/**
 * Encode union address.
 * This is used when `send`ing assets from evm-chain to union.
 */
export function evmEncodeUnionAddress(unionAddress: string) {
  return bytesToHex(fromBech32(unionAddress).data)
}
