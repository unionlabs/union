import * as Effect from "effect/Effect"
import { pipe } from "effect/Function"
import * as ParseResult from "effect/ParseResult"
import * as Schema from "effect/Schema"
import * as WasmTest from "./WasmTest.js"

export const FixedBytes_HexPrefixed__32_ = Schema.String.pipe(
  Schema.annotations({
    description: "A string representation of fixed bytes of length 32, encoded via HexPrefixed",
  }),
)
export type FixedBytes_HexPrefixed__32_ = typeof FixedBytes_HexPrefixed__32_

export const U256 = Schema.String.pipe(
  Schema.annotations({
    description: "256-bit unsigned integer, represented as a decimal string",
  }),
)
export type U256 = typeof U256.Type

export const Bytes_HexPrefixed_ = Schema.String.pipe(
  Schema.annotations({
    description: "A string representation of bytes, encoded via HexPrefixed",
  }),
)
export type Bytes_HexPrefixed_ = typeof Bytes_HexPrefixed_.Type

export const ForwardV0Ack = Schema.Struct({})
export type ForwardV0Ack = typeof ForwardV0Ack.Type

export const BatchInstructionV0Ack = Schema.Union(
  Schema.Union(
    Schema.Union(
      Schema.Literal("protocol"),
      Schema.Struct({
        market_maker: Schema.Struct({ market_maker: Bytes_HexPrefixed_ }),
      }),
    ),
    Schema.Union(
      Schema.Literal("protocol"),
      Schema.Struct({
        market_maker: Schema.Struct({ market_maker: Bytes_HexPrefixed_ }),
      }),
    ),
  ),
  Schema.Union(
    Schema.Union(
      Schema.Literal("non_eureka"),
      Schema.Struct({ eureka: Bytes_HexPrefixed_ }),
    ),
  ),
)
export type BatchInstructionV0Ack = typeof BatchInstructionV0Ack.Type

export const CallAck = Schema.Union(
  Schema.Union(
    Schema.Literal("non_eureka"),
    Schema.Struct({ eureka: Bytes_HexPrefixed_ }),
  ),
)
export type CallAck = typeof CallAck.Type

export const TokenOrderAck = Schema.Union(
  Schema.Union(
    Schema.Literal("protocol"),
    Schema.Struct({
      market_maker: Schema.Struct({ market_maker: Bytes_HexPrefixed_ }),
    }),
  ),
  Schema.Union(
    Schema.Literal("protocol"),
    Schema.Struct({
      market_maker: Schema.Struct({ market_maker: Bytes_HexPrefixed_ }),
    }),
  ),
)
export type TokenOrderAck = typeof TokenOrderAck

export const ForwardAck = Schema.Union(
  Schema.Struct({ v0: ForwardV0Ack }),
)
export type ForwardAck = typeof ForwardAck.Type

export const BatchAck = Schema.Union(
  Schema.Struct({
    "@version": Schema.Literal("v0"),
    "acknowledgements": Schema.Array(BatchInstructionV0Ack),
  }),
)
export type BatchAck = typeof BatchAck.Type

export const RootAck = Schema.Union(
  Schema.Struct({ batch: BatchAck }),
  Schema.Struct({ token_order: TokenOrderAck }),
  Schema.Struct({ call: CallAck }),
  Schema.Struct({ forward: ForwardAck }),
)
export type RootAck = typeof RootAck.Type

export const Ack = Schema.Union(
  Schema.Struct({ success: RootAck }),
  Schema.Struct({ failure: Bytes_HexPrefixed_ }),
).pipe(
  Schema.annotations({ title: "Ack" }),
)
export type Ack = typeof Ack.Type

export const TokenOrderV2Metadata = Schema.Union(
  Schema.Struct({
    "@kind": Schema.Literal("initialize"),
    "implementation": Bytes_HexPrefixed_,
    "initializer": Bytes_HexPrefixed_,
  }),
  Schema.Struct({
    "@kind": Schema.Literal("escrow"),
    "data": Bytes_HexPrefixed_,
  }),
  Schema.Struct({
    "@kind": Schema.Literal("unescrow"),
    "data": Bytes_HexPrefixed_,
  }),
  Schema.Struct({
    "@kind": Schema.Literal("solve"),
    "metadata": Bytes_HexPrefixed_,
    "solver_address": Bytes_HexPrefixed_,
  }),
)
export type TokenOrderV2Metadata = typeof TokenOrderV2Metadata.Type

export const BatchInstructionV0 = Schema.Union(
  Schema.Union(
    Schema.Struct({
      "@version": Schema.Literal("v1"),
      "@opcode": Schema.String, // XXX: replace with string literal union
      "base_amount": U256,
      "base_token": Bytes_HexPrefixed_,
      "base_token_decimals": Schema.Number.pipe(
        Schema.annotations({ format: "uint8" }),
        Schema.int(),
        Schema.greaterThan(0),
      ),
      "base_token_name": Schema.String,
      "base_token_path": U256,
      "base_token_symbol": Schema.String,
      "quote_amount": U256,
      "quote_token": Bytes_HexPrefixed_,
      "receiver": Bytes_HexPrefixed_,
      "sender": Bytes_HexPrefixed_,
    }),
    Schema.Struct({
      "@version": Schema.Literal("v2"),
      "@opcode": Schema.String, // XXX: replace with string literal union
      "base_amount": U256,
      "base_token": Bytes_HexPrefixed_,
      "metadata": TokenOrderV2Metadata,
      "quote_amount": U256,
      "quote_token": Bytes_HexPrefixed_,
      "receiver": Bytes_HexPrefixed_,
      "sender": Bytes_HexPrefixed_,
    }),
  ),
  Schema.Union(
    Schema.Struct({
      "@version": Schema.Literal("v0"),
      "@opcode": Schema.String, // XXX: replace with string literal union
      "contract_address": Bytes_HexPrefixed_,
      "contract_calldata": Bytes_HexPrefixed_,
      "eureka": Schema.Boolean,
      "sender": Bytes_HexPrefixed_,
    }),
  ),
)
export type BatchInstructionV0 = typeof BatchInstructionV0.Type

export type Root =
  // TODO: replace with encoded types as union members are extracted
  | {
    readonly "@version": "v0"
    readonly "@opcode": "batch"
    readonly "instructions": ReadonlyArray<BatchInstructionV0>
  }
  | (
    | {
      readonly "@version": "v1"
      readonly "@opcode": "token_order"
      readonly "base_amount": string
      readonly "base_token": string
      readonly "base_token_decimals": number
      readonly "base_token_name": string
      readonly "base_token_path": string
      readonly "base_token_symbol": string
      readonly "quote_amount": string
      readonly "quote_token": string
      readonly "receiver": string
      readonly "sender": string
    }
    | {
      readonly "@version": "v2"
      readonly "@opcode": "token_order"
      readonly "base_amount": string
      readonly "base_token": string
      readonly "metadata": TokenOrderV2Metadata
      readonly "quote_amount": string
      readonly "quote_token": string
      readonly "receiver": string
      readonly "sender": string
    }
  )
  | {
    readonly "@version": "v0"
    readonly "@opcode": "call"
    readonly "contract_address": string
    readonly "contract_calldata": string
    readonly "eureka": boolean
    readonly "sender": string
  }
  | {
    readonly "@version": "v0"
    readonly "instruction": Root
    readonly "path": string
    readonly "timeout_height": number
    readonly "timeout_timestamp": number
  }

export const Root = Schema.Union(
  Schema.Union(
    Schema.Struct({
      "@version": Schema.Literal("v0"),
      "@opcode": Schema.Literal("batch"),
      "instructions": Schema.Array(BatchInstructionV0),
    }),
  ),
  Schema.Union(
    Schema.Struct({
      "@version": Schema.Literal("v1"),
      "@opcode": Schema.Literal("token_order"),
      "base_amount": U256,
      "base_token": Bytes_HexPrefixed_,
      "base_token_decimals": Schema.Number.pipe(
        Schema.annotations({ format: "uint8" }),
        Schema.int(),
        Schema.greaterThan(0),
      ),
      "base_token_name": Schema.String,
      "base_token_path": U256,
      "base_token_symbol": Schema.String,
      "quote_amount": U256,
      "quote_token": Bytes_HexPrefixed_,
      "receiver": Bytes_HexPrefixed_,
      "sender": Bytes_HexPrefixed_,
    }),
    Schema.Struct({
      "@version": Schema.Literal("v2"),
      "@opcode": Schema.Literal("token_order"),
      "base_amount": U256,
      "base_token": Bytes_HexPrefixed_,
      "metadata": TokenOrderV2Metadata,
      "quote_amount": U256,
      "quote_token": Bytes_HexPrefixed_,
      "receiver": Bytes_HexPrefixed_,
      "sender": Bytes_HexPrefixed_,
    }),
  ),
  Schema.Union(
    Schema.Struct({
      "@version": Schema.Literal("v0"),
      "@opcode": Schema.Literal("call"),
      "contract_address": Bytes_HexPrefixed_,
      "contract_calldata": Bytes_HexPrefixed_,
      "eureka": Schema.Boolean,
      "sender": Bytes_HexPrefixed_,
    }),
  ),
  Schema.Union(
    Schema.Struct({
      "@version": Schema.Literal("v0"),
      "instruction": Schema.suspend((): Schema.Schema<Root> => Root),
      "path": U256,
      "timeout_height": Schema.Number.pipe(
        Schema.annotations({ format: "uint64" }),
        Schema.int(),
        Schema.greaterThan(0),
      ),
      "timeout_timestamp": Schema.Number.pipe(
        Schema.annotations({ format: "uint64" }),
        Schema.int(),
        Schema.greaterThan(0),
      ),
    }),
  ),
)

export const ZkgmPacket = Schema.Struct({
  instruction: Root,
  path: U256,
  salt: FixedBytes_HexPrefixed__32_,
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
    decode: (fromA, _options, ast) =>
      pipe(
        WasmTest.WasmTest,
        Effect.andThen((wasm) => wasm.decodePacket(fromA)),
        Effect.mapError((e) => new ParseResult.Type(ast, fromA, e.message)),
      ),
    encode: (toI, _options, ast) =>
      pipe(
        WasmTest.WasmTest,
        Effect.andThen((wasm) => wasm.encodePacket(toI)),
        Effect.mapError((e) => new ParseResult.Type(ast, toI, e.message)),
      ),
  },
)
