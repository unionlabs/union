import { Inspectable } from "effect"
import { NonEmptyReadonlyArray } from "effect/Array"
import * as A from "effect/Array"
import { ParseError } from "effect/ParseResult"
import { Pipeable, pipeArguments } from "effect/Pipeable"
import { ZkgmInstruction } from "./index.js"
import * as internal from "./internal/batch.js"

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
export interface Batch<A>
  extends Inspectable.Inspectable, Pipeable, ZkgmInstruction.Encodeable<ParseError, never>
{
  readonly [TypeId]: TypeId
  _tag: "Batch"
  readonly instructions: NonEmptyReadonlyArray<ZkgmInstruction.ZkgmInstruction>
}

const Proto = {
  [TypeId]: TypeId,
  _tag: "Batch",
  ...Inspectable.BaseProto,
  toJSON(this: Batch<any>): unknown {
    return {
      _id: "@unionlabs/sdk/Batch",
      instructions: A.map(this.instructions, (x) => x.toJSON()),
    }
  },
  pipe() {
    return pipeArguments(this, arguments)
  },
}

export const make = <
  A extends ZkgmInstruction.ZkgmInstruction,
>(iterable: Iterable<A>): Batch<A> =>
  Object.assign(Object.create(Proto), {
    _tag: "Batch",
    instructions: iterable,
  })
