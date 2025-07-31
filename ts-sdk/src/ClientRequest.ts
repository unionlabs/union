import { NonEmptyReadonlyArray } from "effect/Array"
import type { Inspectable } from "effect/Inspectable"
import type { Pipeable } from "effect/Pipeable"
import * as internal from "./internal/clientRequest.js"
import { Chain } from "./schema/chain.js"
import type { Channel } from "./schema/channel.js"
import { Hex } from "./schema/hex.js"
import * as Token from "./Token.js"

/**
 * @category type ids
 * @since 2.0.0
 */
export const TypeId: unique symbol = Symbol.for("@unionlabs/sdk/ClientRequest")

/**
 * @since 2.0.0
 * @category type ids
 */
export type TypeId = typeof TypeId

/**
 * @since 2.0.0
 * @category models
 */
export interface ClientRequest extends Inspectable, Pipeable {
  readonly [TypeId]: TypeId
  readonly method: Method
  readonly source: Chain
  readonly destination: Chain
  readonly channel: Channel
  readonly sender: string
  readonly receiver: string
  readonly amount: bigint
  readonly fees: NonEmptyReadonlyArray<{
    token: Token.Any
    amount: bigint
  }>
  readonly baseToken: Token.Any | string
  readonly quoteToken: Token.Any | string | "auto"
  readonly metadata: Hex
}

/**
 * @since 2.0.0
 * @category models
 */
export interface Options {
  readonly method?: Method | undefined
  readonly source?: Chain | undefined
  readonly destination?: Chain | undefined
  readonly channel?: Channel | undefined
  readonly sender?: string | undefined
  readonly receiver?: string | undefined
  readonly amount?: bigint | undefined
  readonly baseToken?: Token.Any | string | undefined
  readonly quoteToken?: Token.Any | string | "auto" | undefined
  readonly metadata?: Hex | undefined
}

/**
 * @since 2.0.0
 */
export declare namespace Options {
  /**
   * @category models
   * @since 2.0.0
   */
  export interface Send extends Omit<Options, "method" | "sender" | "receiver"> {}
}

/**
 * @category constructors
 * @since 2.0.0
 */
export const make: <M extends Method>(
  method: M,
) => (
  source: Chain,
  destination: Chain,
  sender: string,
  receiver: string,
  options?: (M extends "SEND" ? Options.Send : Options.NoQuote) | undefined,
) => ClientRequest = internal.make

/**
 * @category constructors
 * @since 2.0.0
 */
export const send: (sender: string, receiver: string, options?: Options.Send) => ClientRequest =
  internal.send

/**
 * @category combinators
 * @since 2.0.0
 */
export const setSender: {
  (sender: string): (self: ClientRequest) => ClientRequest
  (self: ClientRequest, sender: string): ClientRequest
} = internal.setSender

/**
 * @category combinators
 * @since 2.0.0
 */
export const setReceiver: {
  (receiver: string): (self: ClientRequest) => ClientRequest
  (self: ClientRequest, receiver: string): ClientRequest
} = internal.setSender
