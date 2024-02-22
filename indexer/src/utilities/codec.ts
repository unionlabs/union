import { bytesToHex, type Address } from 'viem'
import { fromBech32, toBech32 } from '@cosmjs/encoding'

/**
 * Get union address from ethereum address
 */
export function evmDecodeUnionAddress(ethereumAddress: string) {
  const addressToBuffer = Buffer.from(ethereumAddress.slice(2), 'hex')
  return toBech32('union', addressToBuffer)
}
