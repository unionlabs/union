import { Data, Schema as S } from "effect"
import type { NonEmptyReadonlyArray } from "effect/Array"
import { encodeAbiParameters } from "viem"
import { batchAbi, forwardAbi, fungibleAssetOrderAbi, multiplexAbi } from "../evm/abi/index.js"
import { Hex, HexChecksum } from "../schema/hex.js"

const Version = S.NonNegativeInt
type Version = typeof Version.Type

const OpCode = S.NonNegativeInt
type OpCode = typeof OpCode.Type

const Operand = S.Union(
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
  // [`0x${string}`, boolean, `0x${string}`, `0x${string}`]
  S.Tuple(Hex, S.Boolean, Hex, Hex),
  // [readonly { version: number; opcode: number; operand: `0x${string}`; }[]]
  S.Tuple(S.Array(S.Struct({ version: Version, opcode: OpCode, operand: Hex }))),
  // [`0x${string}`, `0x${string}`, `0x${string}`, bigint, string, string, number, bigint, `0x${string}`, bigint]
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
  // [bigint, `0x${string}`]
  S.Tuple(S.BigIntFromSelf, Hex),
  // [readonly `0x${string}`[]]
  S.Tuple(S.NonEmptyArray(Hex)),
)
type Operand = typeof Operand.Type

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
  operand: Operand.pipe(S.itemsCount(4)),
}) {}

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
  operand: Operand,
}) {}

export type Schema = Forward | Multiplex | Batch | FungibleAssetOrder

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
    readonly operand: NonEmptyReadonlyArray<SchemaEncoded>
  }
  | typeof FungibleAssetOrder.Encoded

export const Schema = S.Union(Forward, Multiplex, Batch, FungibleAssetOrder)

export const Instruction = Data.taggedEnum<Instruction>()
export type Instruction = typeof Schema.Type

export const {
  $match: match,
  $is: is,
  Forward: ForwardRaw,
  Multiplex: MultiplexRaw,
  Batch: BatchRaw,
  FungibleAssetOrder: FungibleAssetOrderRaw,
} = Instruction

export const encodeAbi: (_: Instruction) => Hex = Instruction.$match({
  Forward: ({ operand }) =>
    encodeAbiParameters(forwardAbi, [
      operand[0],
      operand[1],
      operand[2],
      {
        opcode: operand[3].opcode,
        version: operand[3].version,
        operand: encodeAbi(operand[3]),
      },
    ]),
  Multiplex: ({ operand }) => encodeAbiParameters(multiplexAbi, operand),
  Batch: ({ operand }) =>
    encodeAbiParameters(batchAbi, [
      operand.map((i: Schema) => ({
        version: i.version,
        opcode: i.opcode,
        operand: encodeAbi(i),
      })),
    ]),
  FungibleAssetOrder: ({ operand }) => encodeAbiParameters(fungibleAssetOrderAbi, operand),
})
