import type { Effect, Schema as S } from "effect"
import type * as Batch from "./Batch.js"
import { Hex } from "./schema/hex.js"
import type * as Token from "./Token.js"
import * as TokenOrder from "./TokenOrder.js"

export type ZkgmInstruction =
  // | Forward
  // | Call
  | Batch.Batch
  | TokenOrder.TokenOrder

export interface Encodeable<E, R> {
  opcode: number
  version: number
  encode: Effect.Effect<Hex, E, R>
}

export declare const requiredBaseTokens: (i: ZkgmInstruction) => ReadonlyArray<Token.Any>
