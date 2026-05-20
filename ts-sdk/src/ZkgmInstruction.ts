/**
 * This module describes high-level requirements for UCS03 instruction, prepared for {@link ZkgmClientRequest}.
 *
 * @since 2.0.0
 */

import type * as Effect from "effect/Effect"
import * as Match from "effect/Match"
import * as Batch from "./Batch.js"
import * as Call from "./Call.js"
import * as TokenOrder from "./TokenOrder.js"
import type * as Ucs03Ng from "./Ucs03Ng.js"
import type * as ZkgmWasm from "./ZkgmWasm.js"

import { pipe } from "effect/Function"
import { ParseError } from "effect/ParseResult"

/**
 * @category models
 * @since 2.0.0
 */
export type ZkgmInstruction =
  // | Forward
  | Call.Call
  | Batch.Batch
  | TokenOrder.TokenOrder

export const encodeNg = (
  self: ZkgmInstruction,
): Effect.Effect<
  Ucs03Ng.Batch | Ucs03Ng.Call | Ucs03Ng.TokenOrder,
  ZkgmWasm.WasmError | ParseError,
  ZkgmWasm.ZkgmWasm
> =>
  pipe(
    Match.value(self),
    Match.tagsExhaustive({
      Batch: Batch.encodeNg,
      Call: Call.encodeNg,
      TokenOrder: TokenOrder.encodeNg,
    }),
  )
