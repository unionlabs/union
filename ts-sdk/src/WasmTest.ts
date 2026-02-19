import * as FileSystem from "@effect/platform/FileSystem"
import * as Context from "effect/Context"
import * as Effect from "effect/Effect"
import * as Layer from "effect/Layer"

const wasmModule = Effect.tryPromise(
  () => import("./internal/wasm/ucs03-zkgm-packet.js"),
)

type WasmModule = typeof import("./internal/wasm/ucs03-zkgm-packet.js")

const wasmUrl = new URL(
  "./internal/wasm/ucs03-zkgm-packet_bg.wasm",
  import.meta.url,
)

export class WasmTest extends Context.Tag("WasmTest")<
  WasmTest,
  {
    decodePacket: (packet: Uint8Array<ArrayBufferLike>) => Effect.Effect<any, any, never>
    encodePacket: (packet: any) => Effect.Effect<any, any, never>
    decodeAck: (shape: any, ack: Uint8Array<ArrayBufferLike>) => Effect.Effect<any, any, never>
    encodeAck: (ack: any) => Effect.Effect<any, any, never>
    decodeInstruction: (instruction: Uint8Array<ArrayBufferLike>) => Effect.Effect<any, any, never>
    encodeInstruction: (instruction: any) => Effect.Effect<any, any, never>
    packetShape: (instruction: any) => Effect.Effect<any, any, never>
  }
>() {}

const make = (
  wasm: WasmModule,
) => ({
  decodePacket: Effect.fn("decodePacket")(
    (packet: Uint8Array) => Effect.try(() => wasm.decode_packet(packet)),
  ),
  encodePacket: Effect.fn("encodePacket")(
    (packet: any) => Effect.try(() => wasm.encode_packet(packet)),
  ),
  decodeAck: Effect.fn("decodePacket")(
    (shape: any, ack: Uint8Array) => Effect.try(() => wasm.decode_ack(shape, ack)),
  ),
  encodeAck: Effect.fn("encodePacket")(
    (ack: any) => Effect.try(() => wasm.encode_ack(ack)),
  ),
  decodeInstruction: Effect.fn("encodePacket")(
    (instruction: Uint8Array) => Effect.try(() => wasm.decode_instruction(instruction)),
  ),
  encodeInstruction: Effect.fn("encodePacket")(
    (instruction: any) => Effect.try(() => wasm.encode_instruction(instruction)),
  ),
  packetShape: Effect.fn("encodePacket")(
    (instruction: any) => Effect.try(() => wasm.packet_shape(instruction)),
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
    yield* Effect.tryPromise(() => wasm.default({ module_or_path: bytes }))
    return make(wasm)
  }),
)
// function isNodeRuntime(): boolean {
//   return typeof process !== "undefined" && !!process.versions?.node
// }

// export class WasmTest extends Effect.Service<WasmTest>()("WasmTest", {
//   scoped: Effect.fn(function*(workerUrl?: string) {
//     const wasm = yield* Effect.tryPromise(() => import("./internal/wasm/ucs03-zkgm-packet.js"))

//     yield* Effect.log({ wasm })

//     const wasmUrl = workerUrl ?? new URL(
//       "./internal/wasm/ucs03-zkgm-packet_bg.wasm",
//       import.meta.url,
//     )

//     if (isNodeRuntime()) {
//       const { readFile } = yield* Effect.tryPromise(() => import("node:fs/promises"))
//       const bytes = yield* Effect.tryPromise(() => readFile(wasmUrl))
//       yield* Effect.tryPromise(() => wasm.default({ module_or_path: bytes }))
//     } else {
//       console.log("TRYING TO INSTANTIATE IN BROWSER")
//       yield* Effect.tryPromise(() => wasm.default(workerUrl))
//     }

//     const decodePacket = Effect.fn("decodePacket")(
//       (packet: Uint8Array) => Effect.try(() => wasm.decode_packet(packet)),
//     )

//     const encodePacket = Effect.fn("encodePacket")(
//       (packet: Uint8Array) => Effect.try(() => wasm.encode_packet(packet)),
//     )

//     return {
//       wasm,
//       decodePacket,
//       encodePacket,
//     } as const
//   }),
// }) {}
