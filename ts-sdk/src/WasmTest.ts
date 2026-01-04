import { Effect } from "effect"

function isNodeRuntime(): boolean {
  return typeof process !== "undefined" && !!process.versions?.node
}

export class WasmTest extends Effect.Service<WasmTest>()("WasmTest", {
  scoped: Effect.fn(function*(workerUrl?: string) {
    const wasm = yield* Effect.tryPromise(() => import("./internal/wasm/ucs03-zkgm-packet.js"))

    yield* Effect.log({ wasm })

    const wasmUrl = workerUrl ?? new URL(
      "./internal/wasm/ucs03-zkgm-packet_bg.wasm",
      import.meta.url,
    )

    if (isNodeRuntime()) {
      const { readFile } = yield* Effect.tryPromise(() => import("node:fs/promises"))
      const bytes = yield* Effect.tryPromise(() => readFile(wasmUrl))
      yield* Effect.tryPromise(() => wasm.default({ module_or_path: bytes }))
    } else {
      console.log("TRYING TO INSTANTIATE IN BROWSER")
      yield* Effect.tryPromise(() => wasm.default(workerUrl))
    }

    const decodePacket = Effect.fn("decodePacket")(
      (packet: Uint8Array) => Effect.try(() => wasm.decode_packet(packet)),
    )

    const encodePacket = Effect.fn("encodePacket")(
      (packet: Uint8Array) => Effect.try(() => wasm.encode_packet(packet)),
    )

    return {
      wasm,
      decodePacket,
      encodePacket,
    } as const
  }),
}) {}
