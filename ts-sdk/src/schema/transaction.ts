import * as S from "effect/Schema"
import { Hex } from "./hex.js"
/**
 * A branded type for transaction hashes
 */
export const TransactionHash = Hex.pipe(S.brand("TransactionHash"))
export type TransactionHash = typeof TransactionHash.Type
