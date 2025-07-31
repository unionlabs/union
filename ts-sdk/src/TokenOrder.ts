import { Effect, ParseResult } from "effect"
import type { Inspectable } from "effect/Inspectable"
import type { Pipeable } from "effect/Pipeable"
import * as S from "effect/Schema"
import { Covariant } from "effect/Types"
import * as Batch from "./Batch.js"
import { ChannelRegistry } from "./ChannelRegistry.js"
import { ZkgmInstruction } from "./index.js"
import * as internal from "./internal/tokenOrder.js"
import { Chain } from "./schema/chain.js"
import { Hex } from "./schema/hex.js"
import * as Token from "./Token.js"
import * as Ucs05 from "./Ucs05.js"

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

export enum Kind {
  Initialize,
  Escrow,
  Unescrow,
}

export const Input = S.Struct({
  source: Chain,
  destination: Chain,
  sender: Ucs05.ValidAddress,
  receiver: Ucs05.ValidAddress,
  baseToken: S.Union(Token.Any, Token.TokenFromString),
  baseAmount: S.BigIntFromSelf,
  quoteToken: S.Union(Token.Any, Token.TokenFromString),
  quoteAmount: S.BigIntFromSelf,
  kind: S.Enums(Kind),
  metadata: S.optional(Hex), // TODO: default to none
})
export type InputEncoded = typeof Input.Encoded
export type InputDecoded = typeof Input.Type
const Options = S.partial(Input)
type Options = typeof Options.Encoded

/**
 * @since 2.0.0
 * @category models
 */
export interface TokenOrder
  extends
    TokenOrder.Variance<never>,
    Inspectable,
    Pipeable,
    InputDecoded,
    ZkgmInstruction.Encodeable<any, never>
{
  _tag: "TokenOrder"
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

  export type Complete = Build<never>
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
  P extends Options.Required & Partial<Options.Optional>,
>(
  value: P,
) => Effect.Effect<
  TokenOrder.Build<Exclude<keyof Options.Optional, keyof P>>,
  ParseResult.ParseError,
  never
>

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

export declare const withAutoQuoteToken: <A extends keyof Options.Optional>(
  a: TokenOrder.Build<A | "quoteToken">,
) => Effect.Effect<TokenOrder.Build<Exclude<A, "quoteToken">>, never, "quote registry">

/**
 * correct to calc fee based on channel
 */
export declare const withFee: (
  options?: {
    priority: "low" | "average" | "high"
  } | undefined,
) => <A extends TokenOrder.Complete>(
  self: A,
) => Effect.Effect<Batch.Batch<TokenOrder.Complete>, unknown, "with fee" | ChannelRegistry>
