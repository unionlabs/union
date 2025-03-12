import { Schema } from "effect"
import { Hex } from "./hex.ts"

/**
 * A branded type for transaction hashes
 */
export const TransactionHash = Hex.pipe(Schema.brand("TransactionHash"))
export type TransactionHash = typeof TransactionHash.Type
