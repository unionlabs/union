import type { Brand, Option } from "effect"
import type { Inspectable } from "effect/Inspectable"
import type { Pipeable } from "effect/Pipeable"
import type { ReadonlyRecord } from "effect/Record"
import * as internal from "./internal/clientRequest.js"
import type { Channel } from "./schema/channel.js"
import * as Token from "./Token.js"

/**
 * @category type ids
 * @since 2.0.0
 */
export const TypeId: unique symbol = Symbol.for("@unionlabs/sdk/ClientRequest")

/**
 * @since 1.0.0
 * @category type ids
 */
export type TypeId = typeof TypeId

/**
 * TODO: move into own module
 * @since 1.0.0
 * @category models
 */
export type Method = "SEND"

/**
 * @since 1.0.0
 * @category models
 */
export interface ClientRequest extends Inspectable, Pipeable {
  readonly [TypeId]: TypeId
  readonly method: Method
  readonly sender: string
  readonly receiver: string
  readonly amount: bigint
  readonly baseToken: Token.Any | string
  readonly quoteToken: Token.Any | string | "auto"
}

/**
 * @since 1.0.0
 * @category models
 */
export interface Options {
  /** Type of transfer */
  readonly method?: Method | undefined
  /** Address to send from. */
  readonly sender?: string | undefined
  /** Base token */
  readonly baseToken?: string | undefined
  /** Quote tooken. */
  readonly quoteToken?: string | undefined
  /** Address to send from. */
  readonly receiver?: string | undefined
  /** Fee priority for gas calculation. */
  readonly feePriority?: "low" | "average" | "high" | undefined
  /** Explicit channel. */
  readonly channel?: Channel
  /** Batch this request with others. */
  readonly batch?: boolean | undefined
  readonly meta?: ReadonlyRecord<string, unknown>
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

  /**
   * @category models
   * @since 2.0.0
   */
  export interface NoQuote extends Omit<Options, "method" | "sender" | "receiver" | "quoteToken"> {}
}

/**
 * @category constructors
 * @since 2.0.0
 */
export const make: <M extends Method>(
  method: M,
) => (
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
