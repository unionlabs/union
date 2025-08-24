/**
 * This module describes high-level requirements for UCS03 instruction, prepared for {@link ZkgmClientRequest}.
 *
 * @since 2.0.0
 */

// import type { Effect } from "effect"
import type * as Batch from "./Batch.js"
// import { Hex } from "./schema/hex.js"
import type * as Token from "./Token.js"
import * as TokenOrder from "./TokenOrder.js"

/**
 * @category models
 * @since 2.0.0
 */
export type ZkgmInstruction =
  // | Forward
  // | Call
  | Batch.Batch
  | TokenOrder.TokenOrder

/**
 * @category models
 * @since 2.0.0
 */
// export interface Encodeable<E, R> {
//   opcode: number
//   version: number
//   encode: Effect.Effect<Hex, E, R>
// }

/**
 * @category utils
 * @since 2.0.0
 */
export declare const requiredBaseTokens: (i: ZkgmInstruction) => ReadonlyArray<Token.Any>
