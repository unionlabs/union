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

/** @since 2.0.0 */
export namespace Transport {
  export interface Sui {
    readonly relayStoreId: string
    readonly vaultId: string
    readonly ibcStoreId: string
    /** One or more coins a user wants to spend. Keep array for multi-coin support. */
    readonly coins: ReadonlyArray<{
      /** e.g. "0x2::sui::SUI" or a custom coin type */
      readonly typeArg: string
      /** Concrete coin object id(s) for spending */
      readonly objectId: string
    }>
  }

  export interface Evm {
    readonly _?: never
  }

  export interface Cosmos {
    readonly _?: never
  }

  export interface Params {
    readonly sui?: Sui | undefined
    readonly evm?: Evm | undefined
    readonly cosmos?: Cosmos | undefined
  }
}

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
  /** NEW: optional, per-runtime parameters (non-breaking) */
  readonly transport?: Transport.Params | undefined
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
  /** NEW: optional, per-runtime parameters (non-breaking) */
  readonly transport?: Transport.Params | undefined
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
  transport?: Transport.Params | undefined
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
