/**
 * This module interfaces with the indexer via the GraphQL protocol.
 *
 * @since 2.0.0
 */
import { flow, identity, Inspectable, Match, pipe } from "effect"
import { NonEmptyReadonlyArray } from "effect/Array"
import * as A from "effect/Array"
import * as TokenOrder from "./TokenOrder.js"
// import { ParseError } from "effect/ParseResult"
import { Pipeable, pipeArguments } from "effect/Pipeable"
// import * as Schema from "effect/Schema"
import * as O from "effect/Option"
import { ZkgmInstruction } from "./index.js"
import * as internal from "./internal/batch.js"
// import { Hex } from "./schema/hex.js"
// import * as Ucs03 from "./Ucs03.js"

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
  extends Inspectable.Inspectable, Pipeable, Iterable<ZkgmInstruction.ZkgmInstruction> // ZkgmInstruction.Encodeable<ParseError, never>
{
  readonly [TypeId]: TypeId
  readonly _tag: "Batch"
  readonly instructions: NonEmptyReadonlyArray<ZkgmInstruction.ZkgmInstruction>
  readonly opcode: 2
  readonly version: 0
}

// /** @internal */
// const encode = (self: Batch): Effect.Effect<Hex, ParseError, never> =>
//   Effect.gen(function*() {
//     const encodedUcs03 = yield* Effect.all(
//       A.map(self.instructions, x => x.encode),
//     )

//     const decodedUcs03 = yield* Effect.all(
//       A.map(encodedUcs03, x => Schema.decode(Ucs03.Ucs03FromHex)(x)),
//     )

//     console.log({ decodedUcs03 })

//     return yield* pipe(
//       Ucs03.Batch.make({
//         opcode: self.opcode,
//         version: self.version,
//         operand: decodedUcs03,
//       }),
//       (x) => {
//         console.log("batch.fromoperand", x)
//         return x
//       },
//       Schema.encode(Ucs03.Ucs03FromHex),
//       Effect.map(Str.toLowerCase),
//     )
//   })

const Proto = {
  [TypeId]: TypeId,
  _tag: "Batch",
  ...Inspectable.BaseProto,
  [Symbol.iterator](this: Batch) {
    return this.instructions[Symbol.iterator]()
  },
  // encode(this: Batch) {
  //   return encode(this)
  // },
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
  const self = Object.create(Proto)

  self.instructions = iterable
  self.version = 0
  self.opcode = 2
  self._tag = "Batch"
  // self.encode = encode(self)

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
