/**
 * This module interfaces with the indexer via the GraphQL protocol.
 *
 * @since 2.0.0
 */
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
 * @category type ids
 * @since 2.0.0
 */
export type TypeId = typeof TypeId

/**
 * @category models
 * @since 2.0.0
 */
export interface Batch
  extends
    Inspectable.Inspectable,
    Pipeable,
    Iterable<ZkgmInstruction.ZkgmInstruction>,
    ZkgmInstruction.Encodeable<ParseError, never>
{
  readonly [TypeId]: TypeId
  _tag: "Batch"
  readonly instructions: NonEmptyReadonlyArray<ZkgmInstruction.ZkgmInstruction>
}

const Proto = {
  [TypeId]: TypeId,
  _tag: "Batch",
  ...Inspectable.BaseProto,
  [Symbol.iterator](this: Batch) {
    return this.instructions[Symbol.iterator]()
  },
  toJSON(this: Batch): unknown {
    return {
      _id: "@unionlabs/sdk/Batch",
      instructions: A.map(this.instructions, (x) => x.toJSON()),
    }
  },
  pipe() {
    return pipeArguments(this, arguments)
  },
}

/**
 * @category utils
 * @since 2.0.0
 */
export const make = <
  A extends ZkgmInstruction.ZkgmInstruction,
>(iterable: Iterable<A>): Batch =>
  Object.assign(Object.create(Proto), {
    _tag: "Batch",
    instructions: iterable,
  })
