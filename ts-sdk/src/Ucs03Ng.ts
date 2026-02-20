import * as Effect from "effect/Effect"
import { pipe } from "effect/Function"
import * as ParseResult from "effect/ParseResult"
import * as Schema from "effect/Schema"
import * as Uint256 from "./schema/uint256.js"
import * as Uint64 from "./schema/uint64.js"
import * as ZkgmWasm from "./ZkgmWasm.js"

const HexPrefixed = (description: string) =>
  Schema.String.pipe(
    Schema.pattern(/^0x([0-9a-f]{2})*$/),
    Schema.annotations({
      description,
      arbitrary: () => (fc) =>
        fc.integer({ min: 0, max: 128 }).chain(n =>
          fc.string({
            unit: fc.constantFrom(..."0123456789abcdef"),
            maxLength: n * 2,
            minLength: n * 2,
          })
        ).map(s => `0x${s}`),
    }),
  )

const FixedHexPrefixed = (bytes: number) => {
  const len = bytes * 2
  return Schema.String.pipe(
    Schema.pattern(new RegExp(`^0x[0-9a-f]{${len}}$`)),
    Schema.annotations({
      description: `A hex-prefixed string of exactly ${bytes} bytes`,
      arbitrary: () => (fc) =>
        fc.string({ unit: fc.constantFrom(..."0123456789abcdef"), maxLength: len, minLength: len })
          .map(s => `0x${s}`),
    }),
  )
}

export const BytesHexPrefixed32 = FixedHexPrefixed(32)
export type BytesHexPrefixed32 = typeof BytesHexPrefixed32.Type

export const BytesHexPrefixed20 = FixedHexPrefixed(20)
export type BytesHexPrefixed20 = typeof BytesHexPrefixed20.Type

export const BytesHexPrefixed = HexPrefixed(
  "A string representation of bytes, encoded via HexPrefixed",
)
export type BytesHexPrefixed = typeof BytesHexPrefixed.Type

export const CallV0Ack = Schema.Union(
  Schema.Struct({
    "@opcode": Schema.Literal("call"),
    "@version": Schema.Literal("v0"),
    "eureka": BytesHexPrefixed,
  }),
  Schema.Struct({
    "@opcode": Schema.Literal("call"),
    "@version": Schema.Literal("v0"),
    "non_eureka": Schema.optionalWith(Schema.Struct({}), { default: () => ({}) }),
  }),
)

/**
 * NOTE: Union order is important given matching behavior with `optionalWith`.
 */
export const CallAck = Schema.Union(
  CallV0Ack,
)
export type CallAck = typeof CallAck.Type

export const TokenOrderV1Ack = Schema.Union(
  Schema.Struct({
    "@opcode": Schema.Literal("token_order"),
    "@version": Schema.Literal("v1"),
    "market_maker": Schema.Struct({ market_maker: BytesHexPrefixed }),
  }),
  Schema.Struct({
    "@opcode": Schema.Literal("token_order"),
    "@version": Schema.Literal("v1"),
    "protocol": Schema.optionalWith(Schema.Struct({}), { default: () => ({}) }),
  }),
)
export type TokenOrderV1Ack = typeof TokenOrderV1Ack.Type

export const TokenOrderV2Ack = Schema.Union(
  Schema.Struct({
    "@opcode": Schema.Literal("token_order"),
    "@version": Schema.Literal("v2"),
    "market_maker": Schema.Struct({ market_maker: BytesHexPrefixed }),
  }),
  Schema.Struct({
    "@opcode": Schema.Literal("token_order"),
    "@version": Schema.Literal("v2"),
    "protocol": Schema.optionalWith(Schema.Struct({}), { default: () => ({}) }),
  }),
)
export type TokenOrderV2Ack = typeof TokenOrderV2Ack.Type

/**
 * NOTE: Union order is important given matching behavior with `optionalWith`.
 */
export const TokenOrderAck = Schema.Union(
  TokenOrderV1Ack,
  TokenOrderV2Ack,
)
export type TokenOrderAck = typeof TokenOrderAck

export const ForwardV0Ack = Schema.Struct({
  "@opcode": Schema.Literal("forward"),
  "@version": Schema.Literal("v0"),
})
export type ForwardV0Ack = typeof ForwardV0Ack.Type

export const ForwardAck = Schema.Union(
  ForwardV0Ack,
)
export type ForwardAck = typeof ForwardAck.Type

export const BatchInstructionV0Ack = Schema.Union(
  TokenOrderAck,
  CallAck,
)
export type BatchInstructionV0Ack = typeof BatchInstructionV0Ack.Type

export const BatchAck = Schema.Union(
  Schema.Struct({
    "@opcode": Schema.Literal("batch"),
    "@version": Schema.Literal("v0"),
    "acknowledgements": Schema.Array(BatchInstructionV0Ack),
  }),
)
export type BatchAck = typeof BatchAck.Type

export const RootAck = Schema.Union(
  BatchAck,
  TokenOrderAck,
  CallAck,
  ForwardAck,
)
export type RootAck = typeof RootAck.Type

export const Ack = Schema.Union(
  Schema.Struct({ success: RootAck }),
  Schema.Struct({ failure: BytesHexPrefixed }),
).pipe(
  Schema.annotations({ title: "Ack" }),
)
export type Ack = typeof Ack.Type

export const TokenOrderV2Metadata = Schema.Union(
  Schema.Struct({
    "@kind": Schema.Literal("initialize"),
    "implementation": BytesHexPrefixed,
    "initializer": BytesHexPrefixed,
  }),
  Schema.Struct({
    "@kind": Schema.Literal("escrow"),
    "data": BytesHexPrefixed,
  }),
  Schema.Struct({
    "@kind": Schema.Literal("unescrow"),
    "data": BytesHexPrefixed,
  }),
  Schema.Struct({
    "@kind": Schema.Literal("solve"),
    "metadata": BytesHexPrefixed,
    "solver_address": BytesHexPrefixed,
  }),
)
export type TokenOrderV2Metadata = typeof TokenOrderV2Metadata.Type

export const TokenOrderV1 = Schema.Struct({
  "@version": Schema.Literal("v1"),
  "@opcode": Schema.Literal("token_order"),
  "base_amount": Uint256.Uint256,
  "base_token": BytesHexPrefixed,
  "base_token_decimals": Schema.Uint8.pipe(
    Schema.int(),
  ),
  "base_token_name": Schema.String,
  "base_token_path": Uint256.Uint256,
  "base_token_symbol": Schema.String,
  "quote_amount": Uint256.Uint256,
  "quote_token": BytesHexPrefixed,
  "receiver": BytesHexPrefixed,
  "sender": BytesHexPrefixed,
})
export type TokenOrderV1 = typeof TokenOrderV1.Type

export const TokenOrderV2 = Schema.Struct({
  "@version": Schema.Literal("v2"),
  "@opcode": Schema.Literal("token_order"),
  "base_amount": Uint256.Uint256,
  "base_token": BytesHexPrefixed,
  "metadata": TokenOrderV2Metadata,
  "quote_amount": Uint256.Uint256,
  "quote_token": BytesHexPrefixed,
  "receiver": BytesHexPrefixed,
  "sender": BytesHexPrefixed,
})
export type TokenOrderV2 = typeof TokenOrderV2.Type

export const TokenOrder = Schema.Union(
  TokenOrderV1,
  TokenOrderV2,
)
export type TokenOrder = typeof TokenOrder.Type

export const CallV0 = Schema.Struct({
  "@version": Schema.Literal("v0"),
  "@opcode": Schema.Literal("call"),
  "contract_address": BytesHexPrefixed,
  "contract_calldata": BytesHexPrefixed,
  "eureka": Schema.Boolean,
  "sender": BytesHexPrefixed,
})
export type CallV0 = typeof CallV0.Type

export const Call = Schema.Union(
  CallV0,
)
export type Call = typeof Call.Type

export const BatchInstructionV0 = Schema.Union(
  TokenOrder,
  Call,
)
export type BatchInstructionV0 = typeof BatchInstructionV0.Type

export const Batch = Schema.Struct({
  "@version": Schema.Literal("v0"),
  "@opcode": Schema.Literal("batch"),
  "instructions": Schema.Array(BatchInstructionV0),
})
export type Batch = typeof Batch.Type

export interface Forward {
  readonly "@version": "v0"
  readonly "@opcode": "forward"
  readonly "instruction": Root
  readonly "path": bigint
  readonly "timeout_height": bigint
  readonly "timeout_timestamp": bigint
}

export interface ForwardEncoded {
  readonly "@version": "v0"
  readonly "@opcode": "forward"
  readonly "instruction": RootEncoded
  readonly "path": string
  readonly "timeout_height": string
  readonly "timeout_timestamp": string
}

export const Forward = Schema.Struct({
  "@version": Schema.Literal("v0"),
  "@opcode": Schema.Literal("forward"),
  "instruction": Schema.suspend((): Schema.Schema<Root, RootEncoded> => Root),
  "path": Uint256.Uint256,
  "timeout_height": Uint64.Uint64,
  "timeout_timestamp": Uint64.Uint64,
})

export type Root = Batch | TokenOrder | Call | Forward
export type RootEncoded =
  | typeof Batch.Encoded
  | typeof TokenOrder.Encoded
  | typeof Call.Encoded
  | ForwardEncoded

export const Root = Schema.Union(
  Batch,
  TokenOrder,
  Call,
  Forward,
)

export const ZkgmPacket = Schema.Struct({
  instruction: Root,
  path: Uint256.Uint256,
  salt: BytesHexPrefixed32,
}).pipe(
  Schema.annotations({ title: "ZkgmPacket" }),
)
export type ZkgmPacket = typeof ZkgmPacket.Type

export const BatchInstructionV0Shape = Schema.Union(
  Schema.Union(
    Schema.Struct({ "@version": Schema.Literal("v1") }),
    Schema.Struct({ "@version": Schema.Literal("v2") }),
  ),
  Schema.Union(Schema.Struct({ "@version": Schema.Literal("v0"), "eureka": Schema.Boolean })),
)
export type BatchInstructionV0Shape = typeof BatchInstructionV0Shape.Type

export const RootShape = Schema.Union(
  Schema.Union(
    Schema.Struct({
      "@version": Schema.Literal("v0"),
      "instructions": Schema.Array(BatchInstructionV0Shape),
    }),
  ),
  Schema.Union(
    Schema.Struct({ "@version": Schema.Literal("v1") }),
    Schema.Struct({ "@version": Schema.Literal("v2") }),
  ),
  Schema.Union(Schema.Struct({ "@version": Schema.Literal("v0"), "eureka": Schema.Boolean })),
  Schema.Literal("v0"),
).pipe(
  Schema.annotations({ title: "RootShape" }),
)
export type RootShape = typeof RootShape.Type

export const ZkgmPacketFromUint8Array = Schema.transformOrFail(
  Schema.Uint8ArrayFromSelf,
  ZkgmPacket,
  {
    decode: (fromA, _options, ast, _fromI) =>
      pipe(
        ZkgmWasm.ZkgmWasm,
        Effect.andThen((wasm) => wasm.decodePacket(fromA)),
        Effect.mapError((e) => new ParseResult.Type(ast, fromA, e.message)),
      ),
    encode: (toI, _options, ast, _toA) =>
      pipe(
        ZkgmWasm.ZkgmWasm,
        Effect.andThen((wasm) => wasm.encodePacket(toI)),
        Effect.mapError((e) => new ParseResult.Type(ast, toI, e.message)),
      ),
  },
)

export const ZkgmPacketFromHex = Schema.compose(
  Schema.Uint8ArrayFromHex,
  ZkgmPacketFromUint8Array,
)

export const AckFromUint8ArrayWithShape = (shape: RootShape) =>
  Schema.transformOrFail(
    Schema.Uint8ArrayFromSelf,
    Ack,
    {
      decode: (fromA, _options, ast, _fromI) =>
        pipe(
          ZkgmWasm.ZkgmWasm,
          Effect.andThen((wasm) => wasm.decodeAck(fromA, shape)),
          Effect.mapError((e) => new ParseResult.Type(ast, fromA, e.message)),
        ),
      encode: (toI, _options, ast, _toA) =>
        pipe(
          ZkgmWasm.ZkgmWasm,
          Effect.andThen((wasm) => wasm.encodeAck(toI)),
          Effect.mapError((e) => new ParseResult.Type(ast, toI, e.message)),
        ),
    },
  )

export const AckFromUint8ArrayWithInstruction = (instruction: Root) =>
  Schema.transformOrFail(
    Schema.Uint8ArrayFromSelf,
    Ack,
    {
      decode: (fromA, _options, ast, _fromI) =>
        pipe(
          ZkgmWasm.ZkgmWasm,
          Effect.andThen((wasm) =>
            pipe(
              instruction,
              Schema.encode(Root),
              Effect.andThen(wasm.packetShape),
              Effect.andThen((shape) => wasm.decodeAck(fromA, shape)),
            )
          ),
          Effect.mapError((e) => new ParseResult.Type(ast, fromA, e.message)),
        ),
      encode: (toI, _options, ast, _toA) =>
        pipe(
          ZkgmWasm.ZkgmWasm,
          Effect.andThen((wasm) => wasm.encodeAck(toI)),
          Effect.mapError((e) => new ParseResult.Type(ast, toI, e.message)),
        ),
    },
  )

export const AckFromHexWithInstruction = (instruction: Root) =>
  Schema.compose(
    Schema.Uint8ArrayFromHex,
    AckFromUint8ArrayWithInstruction(instruction),
  )

export const InstructionFromUint8Array = Schema.transformOrFail(
  Schema.Uint8ArrayFromSelf,
  Root,
  {
    decode: (fromA, _options, ast, _fromI) =>
      pipe(
        ZkgmWasm.ZkgmWasm,
        Effect.andThen((wasm) => wasm.decodeInstruction(fromA)),
        Effect.mapError((e) => new ParseResult.Type(ast, fromA, e.message)),
      ),
    encode: (toI, _options, ast, _toA) =>
      pipe(
        ZkgmWasm.ZkgmWasm,
        Effect.andThen((wasm) => wasm.encodeInstruction(toI)),
        Effect.mapError((e) => new ParseResult.Type(ast, toI, e.message)),
      ),
  },
)
