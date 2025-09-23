import { Ucs05 } from "@unionlabs/sdk"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { Effect, pipe, Schema } from "effect"
import { bytesToHex, encodeAbiParameters, fromHex, keccak256 } from "viem"

const bytecode_base_checksum =
  "0xec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1" as const
const UCS03_ZKGM = Ucs05.CosmosDisplay.make({
  address: "union1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qpe64fh",
})
const canonical_zkgm = Ucs05.anyDisplayToCanonical(UCS03_ZKGM)
const module_hash = "0x120970d812836f19888625587a4606a5ad23cef31c8684e601771552548fc6b9" as const

/**
 * Generate a deterministic Union cosmos address from an EVM address using instantiate2
 * This is used to create the receiver address for cross-chain operations
 */
export const instantiate2 = Effect.fn(
  function*(options: { path: bigint; channel: ChannelId; sender: Ucs05.AnyDisplay }) {
    const sender = yield* Ucs05.anyDisplayToZkgm(options.sender)
    const abi = [
      {
        name: "path",
        type: "uint256",
        internalType: "uint256",
      },
      {
        name: "channelId",
        type: "uint32",
        internalType: "uint32",
      },
      {
        name: "sender",
        type: "bytes",
        internalType: "bytes",
      },
    ] as const

    const salt = yield* pipe(
      Effect.try(() =>
        encodeAbiParameters(
          abi,
          [
            options.path,
            options.channel,
            sender,
          ] as const,
        )
      ),
      Effect.map((encoded) => keccak256(encoded, "bytes")),
    )

    const u64toBeBytes = (n: bigint) => {
      const buffer = new ArrayBuffer(8)
      const view = new DataView(buffer)
      view.setBigUint64(0, n)
      return new Uint8Array(view.buffer)
    }

    const sha256 = Effect.fn((data: any) =>
      Effect.tryPromise(() => globalThis.crypto.subtle.digest("SHA-256", data))
    )

    const address = yield* pipe(
      Uint8Array.from(
        [
          ...fromHex(module_hash, "bytes"),
          ...new TextEncoder().encode("wasm"),
          0,
          ...u64toBeBytes(32n),
          ...fromHex(bytecode_base_checksum, "bytes"),
          ...u64toBeBytes(32n),
          ...fromHex(canonical_zkgm, "bytes"),
          ...u64toBeBytes(32n),
          ...salt,
          ...u64toBeBytes(0n),
        ],
      ),
      sha256,
      Effect.map((r) => new Uint8Array(r)),
      Effect.map(bytesToHex),
      Effect.flatMap(
        Schema.decode(Ucs05.Bech32FromCanonicalBytesWithPrefix("union")),
      ),
    )

    return Ucs05.CosmosDisplay.make({ address })
  },
)
