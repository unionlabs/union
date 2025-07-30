import type { Effect } from "effect"
import type * as Batch from "./Batch.js"
import { Hex } from "./schema/hex.js"
import type * as TokenOrder from "./TokenOrder.js"

export type ZkgmInstruction =
  // | Forward
  // | Call
  | Batch.Batch<ZkgmInstruction>
  | TokenOrder.TokenOrder

export interface Encodeable {
  encode: Effect.Effect<Hex, any, any>
}
