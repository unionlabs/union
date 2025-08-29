/**
 * This module provides a high-level API for UCS03 `Batch` instruction construction.
 *
 * @since 2.0.0
 */
import { flow, identity, Inspectable, Match, pipe } from "effect"
import { NonEmptyReadonlyArray } from "effect/Array"
import * as A from "effect/Array"
import * as O from "effect/Option"
import { Pipeable, pipeArguments } from "effect/Pipeable"
import { ZkgmInstruction } from "./index.js"
import * as internal from "./internal/batch.js"
import * as TokenOrder from "./TokenOrder.js"

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
  extends Inspectable.Inspectable, Pipeable, Iterable<ZkgmInstruction.ZkgmInstruction>
{
  readonly [TypeId]: TypeId
  readonly _tag: "Batch"
  readonly instructions: NonEmptyReadonlyArray<ZkgmInstruction.ZkgmInstruction>
  readonly opcode: 2
  readonly version: 0
}

const BatchProto: Omit<Batch, "instructions" | "opcode" | "version"> = {
  [TypeId]: TypeId,
  _tag: "Batch",
  ...Inspectable.BaseProto,
  [Symbol.iterator](this: Batch) {
    return this.instructions[Symbol.iterator]()
  },
  toJSON(this: Batch): unknown {
    return {
      _id: "@unionlabs/sdk/Batch",
      instructions: this.instructions,
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
>(iterable: Iterable<A>): Batch => {
  const self = Object.create(BatchProto)

  self.instructions = iterable
  self.version = 0
  self.opcode = 2
  self._tag = "Batch"

  return self
}

/**
 * Optimizes a given `Batch` into a `ZkgmInstruction` such that the following rules are applied:
 * - `TokenOrder`s with zero base amount are removed.
 * - TODO: A `Batch` with a singular `TokenOrder` is reduced to the given `TokenOrder`.
 *
 * @category utils
 * @since 2.0.0
 */
export const optimize = (self: Batch) =>
  pipe(
    self.instructions,
    A.filterMap(pipe(
      Match.type<ZkgmInstruction.ZkgmInstruction>(),
      Match.whenAnd(
        TokenOrder.isTokenOrder,
        { baseAmount: (x) => x === 0n },
        O.none<ZkgmInstruction.ZkgmInstruction>,
      ),
      Match.orElse(flow(identity<ZkgmInstruction.ZkgmInstruction>, O.some)),
    )),
    make,
    // reduction
    // Match.type<ZkgmInstruction.ZkgmInstruction[]>(),
    // Match.whenAnd(
    //   Predicate.isTupleOf(1)<ZkgmInstruction.ZkgmInstruction>,
    //   // Predicate.tuple(TokenOrder.isTokenOrder),
    //   ([tokenOrder]) => tokenOrder,
    // ),
    // Match.orElse((instructions: ZkgmInstruction.ZkgmInstruction[]) => make(instructions))
  )
