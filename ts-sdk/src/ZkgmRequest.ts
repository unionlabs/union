import type { Inspectable } from "effect/Inspectable"
import type { Pipeable } from "effect/Pipeable"
import * as internal from "./internal/zkgmRequest.js"
import { Chain } from "./schema/chain.js"
import type * as ZkgmInstruction from "./ZkgmInstruction.js"

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
export interface ZkgmRequest extends Inspectable, Pipeable {
  readonly [TypeId]: TypeId
  readonly source: Chain
  readonly destination: Chain
  readonly instruction: ZkgmInstruction.ZkgmInstruction
}

/**
 * @since 2.0.0
 * @category models
 */
export interface Options {
  readonly source?: Chain | undefined
  readonly destination?: Chain | undefined
  readonly instruction?: ZkgmInstruction.ZkgmInstruction | undefined
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
export declare const make: (options: {
  source: Chain
  destination: Chain
}) => ZkgmRequest

/**
 * @category constructors
 * @since 2.0.0
 */
export const send: (sender: string, receiver: string, options?: Options.Send) => ZkgmRequest =
  internal.send

/**
 * @category combinators
 * @since 2.0.0
 */
export const setSender: {
  (sender: string): (self: ZkgmRequest) => ZkgmRequest
  (self: ZkgmRequest, sender: string): ZkgmRequest
} = internal.setSender

/**
 * @category combinators
 * @since 2.0.0
 */
export const setReceiver: {
  (receiver: string): (self: ZkgmRequest) => ZkgmRequest
  (self: ZkgmRequest, receiver: string): ZkgmRequest
} = internal.setSender
