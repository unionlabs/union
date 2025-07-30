import { Effect } from "effect"
import type { Inspectable } from "effect/Inspectable"
import type { Pipeable } from "effect/Pipeable"
import { Covariant } from "effect/Types"
import * as Batch from "./Batch.js"
import type * as ChannelRegistry from "./ChannelRegistry.js"
import * as internal from "./internal/clientRequest.js"
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
   * @category models
   * @since 2.0.0
   */
  export interface Send extends
    Omit<
      Options,
      | "source"
      | "destination"
      | "sender"
      | "receiver"
      | "baseToken"
      | "baseAmount"
    >
  {}
}

/**
 * @category constructors
 * @since 2.0.0
 */
export declare const make: (
  source: Chain,
  destination: Chain,
  sender: string,
  receiver: string,
  options?: Options.Send | undefined,
) => TokenOrder

/**
 * @category constructors
 * @since 2.0.0
 */
export const send: (sender: string, receiver: string, options?: Options.Send) => TokenOrder =
  internal.send

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

export declare const withAutoQuoteToken: <A extends keyof Options.Send>(
  a: TokenOrder.Build<A | "quoteToken">,
) => Effect.Effect<TokenOrder.Build<Exclude<A, "quoteToken">>, never, "quote registry">

export declare const withFee: (
  options?: {
    priority: "low" | "average" | "high"
  } | undefined,
) => (
  b: TokenOrder,
) => Effect.Effect<Batch.Batch<TokenOrder>, never, ChannelRegistry.ChannelRegistry>

export declare function makePartial<P extends Partial<TokenOrder>>(
  value: P,
): TokenOrder.Build<Exclude<keyof TokenOrder, keyof P>>

const a = make(
  void 0 as unknown as Chain,
  void 0 as unknown as Chain,
  "",
  "",
)
