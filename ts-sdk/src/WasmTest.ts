import * as FileSystem from "@effect/platform/FileSystem"
import * as Context from "effect/Context"
import * as Effect from "effect/Effect"
import { dual, identity, pipe } from "effect/Function"
import * as Layer from "effect/Layer"
import * as Schema from "effect/Schema"
import * as String from "effect/String"
import type * as Ucs03Ng from "./Ucs03Ng.js"

const wasmModule = Effect.tryPromise(
  () => import("./internal/wasm/ucs03-zkgm-packet.js"),
)

type WasmModule = typeof import("./internal/wasm/ucs03-zkgm-packet.js")

const wasmUrl = new URL(
  "./internal/wasm/ucs03-zkgm-packet_bg.wasm",
  import.meta.url,
)

export class WasmError extends Schema.TaggedError<WasmError>("WasmError")("WasmError", {
  message: Schema.String,
  cause: Schema.optional(Schema.Any),
}) {}

/**
 * WARNING: Direct usage of this servivce is unsafe as return types are not validated.
 *          Use the `Ucs03Ng` module instead for improved safety.
 */
export class WasmTest extends Context.Tag("WasmTest")<
  WasmTest,
  {
    decodePacket: (
      packet: Uint8Array<ArrayBufferLike>,
    ) => Effect.Effect<typeof Ucs03Ng.ZkgmPacket.Encoded, WasmError, never>
    encodePacket: (
      packet: typeof Ucs03Ng.ZkgmPacket.Encoded,
    ) => Effect.Effect<Uint8Array<ArrayBufferLike>, WasmError, never>
    decodeAck: {
      (
        shape: typeof Ucs03Ng.RootShape.Encoded,
      ): (
        ack: Uint8Array<ArrayBufferLike>,
      ) => Effect.Effect<typeof Ucs03Ng.Ack.Encoded, never, never>
      (
        ack: Uint8Array<ArrayBufferLike>,
        shape: typeof Ucs03Ng.RootShape.Encoded,
      ): Effect.Effect<typeof Ucs03Ng.Ack.Encoded, WasmError, never>
    }
    encodeAck: (
      ack: typeof Ucs03Ng.Ack.Encoded,
    ) => Effect.Effect<Uint8Array<ArrayBufferLike>, WasmError, never>
    decodeInstruction: (
      instruction: Uint8Array<ArrayBufferLike>,
    ) => Effect.Effect<typeof Ucs03Ng.Root.Encoded, WasmError, never>
    encodeInstruction: (
      instruction: typeof Ucs03Ng.Root.Encoded,
    ) => Effect.Effect<Uint8Array<ArrayBufferLike>, WasmError, never>
    packetShape: (
      instruction: typeof Ucs03Ng.Root.Encoded,
    ) => Effect.Effect<typeof Ucs03Ng.RootShape.Encoded, WasmError, never>
  }
>() {}

const make = (
  mod: WasmModule,
) => ({
  decodePacket: Effect.fn("decodePacket")(
    (packet: Uint8Array<ArrayBufferLike>) =>
      pipe(
        Effect.try({
          try: () => mod.decode_packet(packet),
          catch: (cause) => new WasmError({ message: "could not decode packet", cause }),
        }),
        Effect.map(identity<typeof Ucs03Ng.ZkgmPacket.Encoded>),
      ),
  ),
  encodePacket: Effect.fn("encodePacket")(
    (packet: typeof Ucs03Ng.ZkgmPacket.Encoded) =>
      pipe(
        Effect.try({
          try: () => mod.encode_packet(packet),
          catch: (cause) =>
            new WasmError({
              message: "could not encode packet" + (`${cause}`),
              cause,
            }),
        }),
        Effect.map(String.substring(2)),
        Effect.flatMap(Schema.decode(Schema.Uint8ArrayFromHex)),
        Effect.catchTag("ParseError", (cause) =>
          new WasmError({
            message: cause.message,
            cause,
          })),
      ),
  ),
  decodeAck: dual(2, (ack, shape) =>
    pipe(
      Effect.try({
        try: () => mod.decode_ack(shape, ack),
        catch: (cause) =>
          new WasmError({
            message: "could not encode packet" + (`${cause}`),
            cause,
          }),
      }),
      Effect.map(identity<typeof Ucs03Ng.Ack.Encoded>),
      Effect.withSpan("decodeAck"),
    )),
  encodeAck: Effect.fn("encodeAck")(
    (ack: typeof Ucs03Ng.Ack.Encoded) =>
      pipe(
        Effect.try({
          try: () => mod.encode_ack(ack),
          catch: (cause) =>
            new WasmError({
              message: "could not encode ack" + (`${cause}`),
              cause,
            }),
        }),
        Effect.map(identity<Uint8Array<ArrayBufferLike>>),
      ),
  ),
  decodeInstruction: Effect.fn("decodeInstruction")(
    (instruction: Uint8Array<ArrayBufferLike>) =>
      pipe(
        Effect.try({
          try: () => mod.decode_instruction(instruction),
          catch: (cause) => new WasmError({ message: "could not decode instruction", cause }),
        }),
        Effect.map(identity<typeof Ucs03Ng.Root.Encoded>),
      ),
  ),
  encodeInstruction: Effect.fn("encodeInstruction")(
    (instruction: typeof Ucs03Ng.Root.Encoded) =>
      pipe(
        Effect.try({
          try: () => mod.encode_instruction(instruction),
          catch: (cause) =>
            new WasmError({
              message: "could not encode instruction" + (`${cause}`),
              cause,
            }),
        }),
        Effect.map(identity<Uint8Array<ArrayBufferLike>>),
      ),
  ),
  packetShape: Effect.fn("packetShape")(
    (instruction: typeof Ucs03Ng.Root.Encoded) =>
      pipe(
        Effect.try({
          try: () => mod.packet_shape(instruction),
          catch: (cause) =>
            new WasmError({
              message: "could not determine packet shape" + (`${cause}`),
              cause,
            }),
        }),
        Effect.map(identity<typeof Ucs03Ng.RootShape.Encoded>),
      ),
  ),
})

export const layerBrowser = Layer.scoped(
  WasmTest,
  Effect.gen(function*() {
    const wasm = yield* wasmModule
    yield* Effect.tryPromise(() => wasm.default(wasmUrl))
    return make(wasm)
  }),
)

export const layerPlatform = Layer.scoped(
  WasmTest,
  Effect.gen(function*() {
    const fs = yield* FileSystem.FileSystem
    const wasm = yield* wasmModule
    const bytes = yield* fs.readFile(wasmUrl.pathname)
    yield* Effect.tryPromise(() => wasm.default(bytes))
    return make(wasm)
  }),
)
