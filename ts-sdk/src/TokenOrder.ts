import { Effect, ParseResult } from "effect"
import type { Inspectable } from "effect/Inspectable"
import { ParseError } from "effect/ParseResult"
import type { Pipeable } from "effect/Pipeable"
import * as S from "effect/Schema"
import { Covariant } from "effect/Types"
import * as Batch from "./Batch.js"
import { ChannelRegistry } from "./ChannelRegistry.js"
import { FeeEstimator } from "./FeeEstimator.js"
import { ZkgmInstruction } from "./index.js"
import * as internal from "./internal/tokenOrder.js"
import { Chain } from "./schema/chain.js"
import { Hex } from "./schema/hex.js"
import * as Token from "./Token.js"
import { TokenRegistry } from "./TokenRegistry.js"
import * as Ucs05 from "./Ucs05.js"

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

export enum Kind {
  Initialize,
  Escrow,
  Unescrow,
}

export const Input = S.Struct({
  source: S.typeSchema(Chain),
  destination: S.typeSchema(Chain),
  sender: S.Union(Ucs05.AnyDisplay, Ucs05.AnyDisplayFromString),
  receiver: S.Union(Ucs05.AnyDisplay, Ucs05.AnyDisplayFromString),
  baseToken: S.Union(Token.Any, Token.TokenFromString),
  baseAmount: S.BigIntFromSelf,
  quoteToken: S.Union(Token.Any, Token.TokenFromString),
  quoteAmount: S.BigIntFromSelf,
  kind: S.Enums(Kind),
  metadata: S.optional(Hex),
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
    ZkgmInstruction.Encodeable<ParseError, never>
{
  _tag: "TokenOrder"
  opcode: 3
  version: 2
}

export declare namespace TokenOrder {
  export interface Variance<out M> {
    readonly [TypeId]: VarianceStruct<M>
  }

  /** Enusres `TokenOrder` is covariant over set of missing keys */
  export interface VarianceStruct<out M> {
    readonly _M: Covariant<M>
  }

  /** Collapses to `TokeOnder` when complete, otherwise tracks variance and partiality. */
  // export type Build<M extends keyof Options = never> = [M] extends [never] ? TokenOrder
  //   : Variance<M>
  export interface Build<
    M extends keyof TokenOrder = never,
  > extends Variance<M>, Pipeable {}

  /** Extracts missing keys from a partial `TokenOrder`. */
  export type Missing<T extends Build<any>> = T extends Build<infer M> ? M : never

  /** Alias for a completed `TokenOrder`. */
  export type Complete = Build<never>
}

/**
 * @since 2.0.0
 */
export declare namespace Options {
  export type RequiredKeys =
    | "source"
    | "destination"
    | "sender"
    | "receiver"
    | "baseToken"
    | "baseAmount"

  /**
   * removes the `?` (optional) modifier,
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
export const make: <
  P extends Options.Required & Partial<Options.Optional>,
>(
  value: P,
) => Effect.Effect<
  Exclude<keyof Options, keyof P> extends never ? TokenOrder
    : TokenOrder.Build<Exclude<keyof Options, keyof P>>,
  ParseResult.ParseError,
  never
> = (value) =>
  internal.make(
    value.source,
    value.destination,
    value.sender,
    value.receiver,
    value.baseToken,
    value.baseAmount,
    value.quoteToken,
    value.quoteAmount,
    value.kind,
    value.metadata,
  )
/**
 * @category combinators
 * @since 2.0.0
 */
export const setSender: {
  (sender: Ucs05.AnyDisplay | string): (self: TokenOrder) => Effect.Effect<TokenOrder, ParseError>
  (self: TokenOrder, sender: Ucs05.AnyDisplay | string): Effect.Effect<TokenOrder, ParseError>
} = internal.setSender

/**
 * @category combinators
 * @since 2.0.0
 */
export const setReceiver: {
  (receiver: Ucs05.AnyDisplay | string): (self: TokenOrder) => Effect.Effect<TokenOrder, ParseError>
  (self: TokenOrder, receiver: Ucs05.AnyDisplay | string): Effect.Effect<TokenOrder, ParseError>
} = internal.setReceiver

export declare const withAutoQuoteToken: <A extends keyof Options.Optional>(
  a: TokenOrder.Build<A | "quoteToken">,
) => Effect.Effect<TokenOrder.Build<Exclude<A, "quoteToken">>, never, TokenRegistry>

/**
 * correct to calc fee based on channel
 */
export declare const withFee: (
  options?: {
    priority: "low" | "average" | "high"
  } | undefined,
) => <A extends TokenOrder.Complete>(
  self: A,
) => Effect.Effect<Batch.Batch<TokenOrder>, unknown, FeeEstimator | ChannelRegistry>
