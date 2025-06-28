/**
 * This module handles construction of the UCS03 instruction.
 *
 * @since 2.0.0
 */

import type * as A from "effect/Array"
import * as Data from "effect/Data"
import * as S from "effect/Schema"
import { encodeAbiParameters } from "viem"
import { Hex, HexChecksum } from "./schema/hex.js"
import * as Ucs03 from "./Ucs03.js"

/**
 * @category models
 * @since 2.0.0
 */
const Version = S.NonNegativeInt
/**
 * @category models
 * @since 2.0.0
 */
type Version = typeof Version.Type

/**
 * @category models
 * @since 2.0.0
 */
const OpCode = S.NonNegativeInt
/**
 * @category models
 * @since 2.0.0
 */
type OpCode = typeof OpCode.Type

/**
 * @category models
 * @since 2.0.0
 */
const MultiplexOperand = S.Union(
  S.Tuple(Hex, S.Boolean, Hex, Hex),
)
/**
 * @category models
 * @since 2.0.0
 */
type MultiplexOperand = typeof MultiplexOperand.Type

/**
 * @category models
 * @since 2.0.0
 */
const FungibleAssetOrderOperand = S.Union(
  S.Tuple(
    Hex,
    Hex,
    Hex,
    S.BigIntFromSelf,
    S.String,
    S.String,
    S.Uint8,
    S.BigIntFromSelf,
    HexChecksum,
    S.BigIntFromSelf,
  ),
)
/**
 * @category models
 * @since 2.0.0
 */
type FungibleAssetOrderOperand = typeof FungibleAssetOrderOperand.Type

/**
 * @category models
 * @since 2.0.0
 */
export const Operand = S.Union(
  // [`0x${string}`, bigint, { version: number; opcode: number; operand: `0x${string}`; }]
  S.Tuple(Hex, S.BigIntFromSelf, S.Struct({ version: Version, opcode: OpCode, operand: Hex })),
  // [number, number, `0x${string}`]
  S.Tuple(S.Number, S.Number, Hex),
  // [bigint, bigint, bigint, { version: number; opcode: number; operand: `0x${string}`; }]
  S.Tuple(
    S.BigIntFromSelf,
    S.BigIntFromSelf,
    S.BigIntFromSelf,
    S.Struct({ version: Version, opcode: OpCode, operand: Hex }),
  ),
  MultiplexOperand,
  // [readonly { version: number; opcode: number; operand: `0x${string}`; }[]]
  S.Tuple(S.Array(S.Struct({ version: Version, opcode: OpCode, operand: Hex }))),
  FungibleAssetOrderOperand,
  // [bigint, `0x${string}`]
  S.Tuple(S.BigIntFromSelf, Hex),
  // [readonly `0x${string}`[]]
  S.Tuple(S.NonEmptyArray(Hex)),
)
/**
 * @category models
 * @since 2.0.0
 */
export type Operand = typeof Operand.Type

/**
 * @category models
 * @since 2.0.0
 */
export class Forward extends S.TaggedClass<Forward>()("Forward", {
  opcode: S.Literal(0).pipe(
    S.optional,
    S.withDefaults({
      constructor: () => 0 as const,
      decoding: () => 0 as const,
    }),
  ),
  version: S.Literal(0).pipe(
    S.optional,
    S.withDefaults({
      constructor: () => 0 as const,
      decoding: () => 0 as const,
    }),
  ),
  operand: S.Tuple(
    // TODO(ehegnes): Check bitwidth constraint
    S.PositiveBigIntFromSelf.annotations({
      description: "Path",
    }),
    S.PositiveBigIntFromSelf.annotations({
      description: "Timeout Height",
    }),
    S.PositiveBigIntFromSelf.annotations({
      description: "Timeout Timestamp",
    }),
    S.suspend((): S.Schema<Schema, SchemaEncoded> => Schema),
  ),
}) {}

/**
 * @category models
 * @since 2.0.0
 */
export class Multiplex extends S.TaggedClass<Multiplex>()("Multiplex", {
  opcode: S.Literal(1).pipe(
    S.optional,
    S.withDefaults({
      constructor: () => 1 as const,
      decoding: () => 1 as const,
    }),
  ),
  version: S.Literal(0).pipe(
    S.optional,
    S.withDefaults({
      constructor: () => 0 as const,
      decoding: () => 0 as const,
    }),
  ),
  operand: MultiplexOperand,
}) {}

/**
 * @category models
 * @since 2.0.0
 */
export class Batch extends S.TaggedClass<Batch>()("Batch", {
  opcode: S.Literal(2).pipe(
    S.optional,
    S.withDefaults({
      constructor: () => 2 as const,
      decoding: () => 2 as const,
    }),
  ),
  version: S.Literal(0).pipe(
    S.optional,
    S.withDefaults({
      constructor: () => 0 as const,
      decoding: () => 0 as const,
    }),
  ),
  operand: S.NonEmptyArray(S.suspend((): S.Schema<Schema, SchemaEncoded> => Schema)),
}) {}

/**
 * @category models
 * @since 2.0.0
 */
export class FungibleAssetOrder extends S.TaggedClass<FungibleAssetOrder>()("FungibleAssetOrder", {
  opcode: S.Literal(3).pipe(
    S.optional,
    S.withDefaults({
      constructor: () => 3 as const,
      decoding: () => 3 as const,
    }),
  ),
  version: S.Literal(1).pipe(
    S.optional,
    S.withDefaults({
      constructor: () => 1 as const,
      decoding: () => 1 as const,
    }),
  ),
  operand: FungibleAssetOrderOperand,
}) {}

/**
 * @category models
 * @since 2.0.0
 */
export type Schema = Forward | Multiplex | Batch | FungibleAssetOrder

/**
 * @category models
 * @since 2.0.0
 */
type SchemaEncoded =
  | {
    readonly _tag: "Forward"
    readonly opcode?: 0 | undefined
    readonly version?: 0 | undefined
    readonly operand: readonly [bigint, bigint, bigint, SchemaEncoded]
  }
  | typeof Multiplex.Encoded
  | {
    readonly _tag: "Batch"
    readonly opcode?: 2 | undefined
    readonly version?: 0 | undefined
    readonly operand: A.NonEmptyReadonlyArray<SchemaEncoded>
  }
  | typeof FungibleAssetOrder.Encoded

/**
 * @category models
 * @since 2.0.0
 */
export const Schema = S.Union(Forward, Multiplex, Batch, FungibleAssetOrder)

/**
 * @category models
 * @since 2.0.0
 */
export const Instruction = Data.taggedEnum<Instruction>()
/**
 * @category models
 * @since 2.0.0
 */
export type Instruction = typeof Schema.Type

/**
 * @category utils
 * @since 2.0.0
 */
export const match = Instruction.$match
/**
 * @example
 * import { Instruction } from "@unionlabs/sdk"
 * import * as fc from "effect/FastCheck"
 * import * as Arbitrary from "effect/Arbitrary"
 *
 * const m: Instruction.Multiplex = fc.sample(Arbitrary.make(Instruction.Multiplex), 1)[0]
 * const b: Instruction.Batch = fc.sample(Arbitrary.make(Instruction.Batch), 1)[0]
 *
 * const isMultiplex = Instruction.is("Multiplex")
 *
 * assert.strictEqual(isMultiplex(m), true)
 * assert.strictEqual(isMultiplex(b), false)
 *
 * @category utils
 * @since 2.0.0
 */
export const is = Instruction.$is

/**
 * @category utils
 * @since 2.0.0
 */
export const ForwardRaw = Instruction.Forward
/**
 * @category utils
 * @since 2.0.0
 */
export const MultiplexRaw = Instruction.Multiplex
/**
 * @category utils
 * @since 2.0.0
 */
export const BatchRaw = Instruction.Batch
/**
 * @category utils
 * @since 2.0.0
 */
export const FungibleAssetOrderRaw = Instruction.FungibleAssetOrder

/**
 * @category utils
 * @since 2.0.0
 */
export const encodeAbi: (_: Instruction) => Hex = Instruction.$match({
  Forward: ({ operand }) =>
    encodeAbiParameters(Ucs03.ForwardAbi(), [
      operand[0],
      operand[1],
      operand[2],
      {
        opcode: operand[3].opcode,
        version: operand[3].version,
        operand: encodeAbi(operand[3]),
      },
    ]),
  Multiplex: ({ operand }) => encodeAbiParameters(Ucs03.MultiplexAbi(), operand),
  Batch: ({ operand }) =>
    encodeAbiParameters(Ucs03.BatchAbi(), [
      operand.map((i: Schema) => ({
        version: i.version,
        opcode: i.opcode,
        operand: encodeAbi(i),
      })),
    ]),
  FungibleAssetOrder: ({ operand }) => encodeAbiParameters(Ucs03.FungibleAssetOrderAbi(), operand),
})
