import { NonEmptyReadonlyArray } from "effect/Array"
import { Inspectable } from "effect/Inspectable"
import { Pipeable } from "effect/Pipeable"

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

/**
 * @since 2.0.0
 * @category models
 */
export interface Batch<A> extends Inspectable, Pipeable {
  readonly [TypeId]: TypeId
  _tag: "Batch"
  readonly instructions: NonEmptyReadonlyArray<A>
}
