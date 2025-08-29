/**
 * This module describes high-level requirements for UCS03 instruction, prepared for {@link ZkgmClientRequest}.
 *
 * @since 2.0.0
 */

import type * as Batch from "./Batch.js"
import type * as Call from "./Call.js"
import * as TokenOrder from "./TokenOrder.js"

/**
 * @category models
 * @since 2.0.0
 */
export type ZkgmInstruction =
  // | Forward
  | Call.Call
  | Batch.Batch
  | TokenOrder.TokenOrder
