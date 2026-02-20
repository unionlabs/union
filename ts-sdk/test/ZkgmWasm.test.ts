import * as NodeContext from "@effect/platform-node/NodeContext"
import { assert, describe, it } from "@effect/vitest"
import * as Effect from "effect/Effect"
import * as Either from "effect/Either"
import * as fc from "effect/FastCheck"
import { pipe } from "effect/Function"
import * as Layer from "effect/Layer"
import * as Schema from "effect/Schema"

import * as Ucs03Ng from "@unionlabs/sdk/Ucs03Ng"
import * as ZkgmWasm from "@unionlabs/sdk/ZkgmWasm"
import * as Arbitrary from "effect/Arbitrary"

const ZkgmWasmTest = pipe(
  ZkgmWasm.layerPlatform,
  Layer.provideMerge(NodeContext.layer),
)

const PACKET_HEX =
  `79176e1d5f2779e14b2f5f885bfe7b35e78802643522ce0dad5cac4e4a44271f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000066000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000003a000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002e00000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001e00000000000000000000000000000000000000000000000000000000000002710000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000002600000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000002a00000000000000000000000000000000000000000000000000000000000002710000000000000000000000000000000000000000000000000000000000000001415ee7c367f4232241028c36e720803100757c6e9000000000000000000000000000000000000000000000000000000000000000000000000000000000000003e62626e316d377a72356a77346b397a32327239616a676766347563616c7779377578767539676b7736746e736d7634326c766a706b7761736167656b356700000000000000000000000000000000000000000000000000000000000000000014e53dcec07d16d88e386ae0710e86d9a400f83c31000000000000000000000000000000000000000000000000000000000000000000000000000000000000000442414259000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000007426162796c6f6e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000047562626e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000120000000000000000000000000000000000000000000000000000000000000001415ee7c367f4232241028c36e720803100757c6e9000000000000000000000000000000000000000000000000000000000000000000000000000000000000003e62626e316d377a72356a77346b397a32327239616a676766347563616c7779377578767539676b7736746e736d7634326c766a706b7761736167656b3567000000000000000000000000000000000000000000000000000000000000000000b27b22626f6e64223a7b22616d6f756e74223a223130303030222c2273616c74223a22307833313333303831396135613232336439376163373134663239616535653361646265396565663833383233373830663761393063636536363461626138366565222c226578706563746564223a2239373237222c22726563697069656e74223a2262626e3168637533306461647770686638397533783375366a327a35387233376339616b687866637330227d7d0000000000000000000000000000`

const PACKET_BYTES = Uint8Array.from(Buffer.from(PACKET_HEX, "hex"))

const PACKET_DECODED = {
  salt: "0x79176e1d5f2779e14b2f5f885bfe7b35e78802643522ce0dad5cac4e4a44271f",
  path: 0n,
  instruction: {
    "@opcode": "batch",
    "@version": "v0",
    "instructions": [
      {
        "@opcode": "token_order",
        "@version": "v1",
        "sender": "0x15ee7c367f4232241028c36e720803100757c6e9",
        "receiver":
          "0x62626e316d377a72356a77346b397a32327239616a676766347563616c7779377578767539676b7736746e736d7634326c766a706b7761736167656b3567",
        "base_token": "0xe53dcec07d16d88e386ae0710e86d9a400f83c31",
        "base_amount": 10000n,
        "base_token_symbol": "BABY",
        "base_token_name": "Babylon",
        "base_token_decimals": 6,
        "base_token_path": 1n,
        "quote_token": "0x7562626e",
        "quote_amount": 10000n,
      },
      {
        "@opcode": "call",
        "@version": "v0",
        "sender": "0x15ee7c367f4232241028c36e720803100757c6e9",
        "eureka": false,
        "contract_address":
          "0x62626e316d377a72356a77346b397a32327239616a676766347563616c7779377578767539676b7736746e736d7634326c766a706b7761736167656b3567",
        "contract_calldata":
          "0x7b22626f6e64223a7b22616d6f756e74223a223130303030222c2273616c74223a22307833313333303831396135613232336439376163373134663239616535653361646265396565663833383233373830663761393063636536363461626138366565222c226578706563746564223a2239373237222c22726563697069656e74223a2262626e3168637533306461647770686638397533783375366a327a35387233376339616b687866637330227d7d",
      },
    ],
  },
} as const

const arbAckForInstruction = (instruction: Ucs03Ng.Root): fc.Arbitrary<Ucs03Ng.Ack> => {
  const arbRootAck = (root: Ucs03Ng.Root): fc.Arbitrary<Ucs03Ng.RootAck> => {
    switch (root["@opcode"]) {
      case "batch":
        return (root.instructions.length === 0
          ? fc.constant([])
          : fc.tuple(...root.instructions.map(arbBatchInstructionAck))).map(acks => ({
            batch: { "@version": "v0" as const, "acknowledgements": acks },
          }))
      case "token_order":
        return Arbitrary.make(Ucs03Ng.TokenOrderAck).map(ack => ({ token_order: ack }))
      case "call":
        return Arbitrary.make(Ucs03Ng.CallAck).map(ack => ({ call: ack }))
      case "forward":
        return fc.constant({ forward: { "@version": "v0" as const } })
    }
  }

  const arbBatchInstructionAck = (instr: Ucs03Ng.BatchInstructionV0) =>
    Arbitrary.make(
      instr["@opcode"] === "token_order" ? Ucs03Ng.TokenOrderAck : Ucs03Ng.CallAck,
    ).map(ack => ({ "@opcode": instr["@opcode"] as any, ...ack }))

  return fc.oneof(
    arbRootAck(instruction).map(success => ({ success })),
    Arbitrary.make(Ucs03Ng.BytesHexPrefixed).map(failure => ({ failure })),
  )
}

describe("WasmTest", () => {
  it.layer(ZkgmWasmTest)((it) => {
    it.effect("example packet iso (wasm)", () =>
      Effect.gen(function*() {
        const wasm = yield* ZkgmWasm.ZkgmWasm
        const decoded = yield* wasm.decodePacket(PACKET_BYTES)
        const encoded = yield* wasm.encodePacket(decoded)
        assert.deepStrictEqual(encoded, PACKET_BYTES)
        const expected = yield* Schema.encode(Ucs03Ng.ZkgmPacket)(PACKET_DECODED)
        assert.deepStrictEqual(decoded, expected)
      }))

    it.effect("example packet iso (schema)", () =>
      Effect.gen(function*() {
        const decoded = yield* Schema.decode(Ucs03Ng.ZkgmPacketFromUint8Array)(PACKET_BYTES)
        const encoded = yield* Schema.encode(Ucs03Ng.ZkgmPacketFromUint8Array)(decoded)
        assert.deepStrictEqual(decoded, PACKET_DECODED)
        assert.deepStrictEqual(encoded, PACKET_BYTES)
      }))

    it.effect.prop(
      "ZkgmPacketFromUint8Array roundtrip",
      { packet: Ucs03Ng.ZkgmPacket },
      ({ packet }) =>
        Effect.gen(function*() {
          const encoded = yield* Schema.encode(Ucs03Ng.ZkgmPacketFromUint8Array)(packet)
          const decoded = yield* Schema.decode(Ucs03Ng.ZkgmPacketFromUint8Array)(encoded)
          assert.deepStrictEqual(decoded, packet)
        }),
    )

    it.effect.prop(
      "ZkgmPacketFromHex roundtrip",
      { packet: Ucs03Ng.ZkgmPacket },
      ({ packet }) =>
        Effect.gen(function*() {
          const encoded = yield* Schema.encode(Ucs03Ng.ZkgmPacketFromHex)(packet)
          const decoded = yield* Schema.decode(Ucs03Ng.ZkgmPacketFromHex)(encoded)
          assert.deepStrictEqual(decoded, packet)
        }),
    )

    // Ack roundtrip — needs correlated generation (ack must match instruction shape),
    // so we generate the packet via prop and build matching acks inside the body
    it.effect.prop(
      "ack roundtrip",
      { packet: Ucs03Ng.ZkgmPacket },
      ({ packet }) =>
        Effect.gen(function*() {
          const ackArb = arbAckForInstruction(packet.instruction)
          const [ack] = fc.sample(ackArb, 1)
          const schema = Ucs03Ng.AckFromUint8ArrayWithInstruction(packet.instruction)
          const bytes = yield* Schema.encode(schema)(ack)
          const decoded = yield* Schema.decode(schema)(bytes)
          assert.deepStrictEqual(decoded, ack)
        }),
    )

    it.effect("ack decode asymmetry check", () =>
      Effect.gen(function*() {
        const wasm = yield* ZkgmWasm.ZkgmWasm

        const ack = {
          success: {
            batch: {
              "@version": "v0",
              "acknowledgements": [{
                "@opcode": "token_order",
                "@version": "v1",
                "market_maker": { market_maker: "0xdeadbeef" },
              }],
            },
          },
        } as const

        const shape = {
          "@opcode": "batch",
          "@version": "v0",
          "instructions": [{ "@opcode": "token_order", "@version": "v1" }],
        } as const

        const bytes = yield* wasm.encodeAck(ack)
        const decoded = yield* wasm.decodeAck(bytes, shape)

        assert.deepStrictEqual(decoded, ack)
      }))

    it.effect.prop(
      "ack roundtrip",
      { packet: Ucs03Ng.ZkgmPacket },
      ({ packet }) =>
        Effect.gen(function*() {
          const ack = fc.sample(arbAckForInstruction(packet.instruction), 1)[0]
          const schema = Ucs03Ng.AckFromUint8ArrayWithInstruction(packet.instruction)
          const bytes = yield* Schema.encode(schema)(ack)
          const decoded = yield* Schema.decode(schema)(bytes)
          assert.deepStrictEqual(decoded, ack)
        }),
    )
  })

  it.effect.only("call ack shape check", () =>
    Effect.gen(function*() {
      const wasm = yield* ZkgmWasm.ZkgmWasm

      const shape = {
        "@opcode": "call",
        "@version": "v0",
        "eureka": false,
      } as const

      // Try different call ack shapes
      const attempts = [
        { success: { call: { "@version": "v0", "non_eureka": {} } } },
        { success: { call: { "@version": "v0", "non_eureka": "non_eureka" } } },
        { success: { call: "non_eureka" } },
      ]

      for (const attempt of attempts) {
        const bytes = yield* Effect.either(wasm.encodeAck(attempt))
        if (Either.isLeft(bytes)) {
          console.log("ENCODE FAILED:", JSON.stringify(attempt), String(bytes.left))
        } else {
          const decoded = yield* Effect.either(wasm.decodeAck(bytes.right, shape))
          if (Either.isRight(decoded)) {
            console.log("SUCCESS:", JSON.stringify(attempt))
            console.log("decoded:", JSON.stringify(decoded.right, null, 2))
          } else {
            console.log("DECODE FAILED:", JSON.stringify(attempt), String(decoded.left))
          }
        }
      }
    }))
})
