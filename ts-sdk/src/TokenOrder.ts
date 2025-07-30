import { Effect } from "effect"
import type { Inspectable } from "effect/Inspectable"
import type { Pipeable } from "effect/Pipeable"
import { Covariant } from "effect/Types"
import * as Batch from "./Batch.js"
import type * as ChannelRegistry from "./ChannelRegistry.js"
import * as internal from "./internal/tokenOrder.js"
import { Chain } from "./schema/chain.js"
import type { Channel } from "./schema/channel.js"
import * as Token from "./Token.js"

/**
 * @category type ids
 * @since 2.0.0
 */
export const TypeId: unique symbol = Symbol.for("@unionlabs/sdk/TokenOrder")

/**
 * @since 2.0.0
 * @category type ids
 */
export type TypeId = typeof TypeId

export const enum Type {
  Initialize,
  Escrow,
  Unescrow,
}

/**
 * @since 2.0.0
 * @category models
 */
export interface TokenOrder extends TokenOrder.Variance<never>, Inspectable, Pipeable {
  _tag: "TokenOrder"
  readonly source: Chain
  readonly destination: Chain
  readonly channel: Channel
  readonly sender: string
  readonly receiver: string
  readonly baseToken: Token.Any | string
  readonly baseAmount: bigint
  readonly quoteToken: Token.Any | string | "auto"
  readonly quoteAmount: bigint
  readonly type: Type
  readonly metadata: string
}

/**
 * @since 2.0.0
 * @category models
 */
export interface Options {
  readonly source?: Chain | undefined
  readonly destination?: Chain | undefined
  readonly channel?: Channel | undefined
  readonly sender?: string | undefined
  readonly receiver?: string | undefined
  readonly baseToken?: Token.Any | string | undefined
  readonly baseAmount?: bigint | undefined
  readonly quoteToken?: Token.Any | string | "auto" | undefined
  readonly quoteAmount?: bigint
  readonly type?: Type | undefined
  readonly metadata?: string | undefined
}

export declare namespace TokenOrder {
  export interface Variance<out M> {
    readonly [TypeId]: VarianceStruct<M>
  }

  export interface VarianceStruct<out M> {
    readonly _M: Covariant<M>
  }

  export interface Build<
    M extends keyof TokenOrder = never,
  > extends Variance<M> {}

  export type Missing<T extends Build<any>> = T extends Build<infer M> ? M : never
}

/**
 * @since 2.0.0
 */
export declare namespace Options {
  /**
   * Keys that **must** be present before the order can leave the caller’s
   * hands.  These map directly to domain requirements – rename as needed.
   */
  export type RequiredKeys =
    | "source"
    | "destination"
    | "sender"
    | "receiver"
    | "baseToken"
    | "baseAmount"

  /**
   * Helper mapped type:
   * – removes the `?` (optional) modifier,
   * – keeps `readonly`,
   * – preserves the exact field types.
   */
  export type Required = {
    -readonly [K in RequiredKeys]-?: NonNullable<Options[K]>
  }

  /**
   * Everything **except** the required six keys; stays optional.
   */
  export type Optional = Omit<Options, RequiredKeys>

  /**
   * A fully‑specified options object: required fields present,
   * optional ones still optional.
   */
  export type Complete = Required & Optional
}

/**
 * @category constructors
 * @since 2.0.0
 */
export declare const make: <
  P extends Partial<Options>,
>(
  value: P,
) => TokenOrder.Build<Exclude<keyof Options, keyof P>>

/**
 * @category combinators
 * @since 2.0.0
 */
export const setSender: {
  (sender: string): (self: TokenOrder) => TokenOrder
  (self: TokenOrder, sender: string): TokenOrder
} = internal.setSender

/**
 * @category combinators
 * @since 2.0.0
 */
export const setReceiver: {
  (receiver: string): (self: TokenOrder) => TokenOrder
  (self: TokenOrder, receiver: string): TokenOrder
} = internal.setReceiver

export declare const withAutoQuoteToken: <A extends keyof Options.Required>(
  a: TokenOrder.Build<A | "quoteToken">,
) => Effect.Effect<TokenOrder.Build<Exclude<A, "quoteToken">>, never, "quote registry">

export declare const withFee: (
  options?: {
    priority: "low" | "average" | "high"
  } | undefined,
) => (
  b: TokenOrder,
) => Effect.Effect<Batch.Batch<TokenOrder>, never, ChannelRegistry.ChannelRegistry>
