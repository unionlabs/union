import { toBech32 } from "@cosmjs/encoding"

/**
 * Get union address from ethereum address
 */
export function evmDecodeUnionAddress(ethereumAddress: string) {
  const addressToBuffer = Buffer.from(ethereumAddress.slice(2), "hex")
  return toBech32("union", addressToBuffer)
}
