/**
 * This module prepares cross-chain request submissions.
 *
 * @since 2.0.0
 */
import { Option } from "effect"
import { NonEmptyReadonlyArray } from "effect/Array"
import type { Inspectable } from "effect/Inspectable"
import type { Pipeable } from "effect/Pipeable"
import * as internal from "./internal/zkgmClientRequest.js"
import { Chain } from "./schema/chain.js"
import { ChannelId } from "./schema/channel.js"
import type * as Token from "./Token.js"
import type * as ZkgmInstruction from "./ZkgmInstruction.js"

/**
 * @category type ids
 * @since 2.0.0
 */
export const TypeId: unique symbol = internal.TypeId

/**
 * @since 2.0.0
 * @category type ids
 */
export type TypeId = typeof TypeId

/**
 * @since 2.0.0
 * @category models
 */
export interface ZkgmClientRequest extends Inspectable, Pipeable {
  readonly [TypeId]: TypeId
  readonly source: Chain
  readonly destination: Chain
  readonly channelId: ChannelId
  readonly ucs03Address: string
  readonly instruction: ZkgmInstruction.ZkgmInstruction
  /**
   * **NOTE:** only for EVM submission
   */
  readonly kind: "execute" | "simulateAndExecute"
}

/**
 * @since 2.0.0
 * @category models
 */
export interface Options {
  readonly source?: Chain | undefined
  readonly destination?: Chain | undefined
  readonly channelId: ChannelId
  readonly ucs03Address: string // XXX: narrow
  readonly instruction?: ZkgmInstruction.ZkgmInstruction | undefined
  readonly kind?: "execute" | "simulateAndExecute" | undefined
}

/**
 * @category constructors
 * @since 2.0.0
 */
export const make: (options: {
  source: Chain
  destination: Chain
  channelId: ChannelId
  ucs03Address: string // XXX: narrow
  instruction: ZkgmInstruction.ZkgmInstruction
  kind?: "execute" | "simulateAndExecute" | undefined
}) => ZkgmClientRequest = internal.make

/**
 * @category combinators
 * @since 2.0.0
 */
export const setSource: {
  (source: Chain): (self: ZkgmClientRequest) => ZkgmClientRequest
  (self: ZkgmClientRequest, source: Chain): ZkgmClientRequest
} = internal.setSource

/**
 * @category combinators
 * @since 2.0.0
 */
export const setDestination: {
  (destination: Chain): (self: ZkgmClientRequest) => ZkgmClientRequest
  (self: ZkgmClientRequest, destination: Chain): ZkgmClientRequest
} = internal.setDestination

/**
 * @category combinators
 * @since 2.0.0
 */
export const setKind: {
  (kind: "execute" | "simulateAndExecute"): (self: ZkgmClientRequest) => ZkgmClientRequest
  (self: ZkgmClientRequest, kind: "execute" | "simulateAndExecute"): ZkgmClientRequest
} = internal.setKind

/**
 * Extracts from a {@link ZkgmClientRequest} any {@link Token.Any}s required for execution along each amount.
 * **NOTE:** Amounts are reduced by summation such that the resulting array is {@link Token.Any}s deduplicated.
 *
 * @category utils
 * @since 2.0.0
 */
export const requiredFunds: (
  self: ZkgmClientRequest,
) => Option.Option<NonEmptyReadonlyArray<readonly [token: Token.Any, amount: bigint]>> =
  internal.requiredFunds
