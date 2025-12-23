import * as S from "effect/Schema"
import type { RpcType } from "./chain.js"
import { Base58FromHex, Hex } from "./hex.js"

/**
 * A branded type for transaction hashes (stored as Hex)
 */
export const TransactionHash = Hex.pipe(S.brand("TransactionHash"))
export type TransactionHash = typeof TransactionHash.Type

/**
 * Formats a transaction hash based on the chain's RPC type.
 * For Sui chains, converts hex to base58.
 * For other chains, returns the hex as-is.
 *
 * @param rpcType - The RPC type of the chain (e.g., "sui", "evm", "cosmos", "aptos")
 * @param txHash - The transaction hash in hex format
 * @returns The formatted transaction hash string
 *
 * @example
 * ```ts
 * formatTransactionHash("sui", "0x1234...") // Returns base58 string
 * formatTransactionHash("evm", "0x1234...") // Returns hex string
 * ```
 */
export function formatTransactionHash(rpcType: RpcType, txHash: Hex): string {
  if (rpcType === "sui") {
    // Convert hex to base58 for Sui chains
    const result = S.decodeSync(Base58FromHex)(txHash)
    return result
  }
  // For all other chains (evm, cosmos, aptos), return hex as-is
  return txHash
}
