import type { Inspectable } from "effect/Inspectable"
import type { Pipeable } from "effect/Pipeable"
import * as internal from "./internal/zkgmClientRequest.js"
import { Chain } from "./schema/chain.js"
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
 * @category constructors
 * @since 2.0.0
 */
export declare const make: (options: {
  source: Chain
  destination: Chain
  instruction: ZkgmInstruction.ZkgmInstruction
}) => ZkgmClientRequest

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
