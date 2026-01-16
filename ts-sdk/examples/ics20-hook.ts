// @ts-ignore
if (typeof BigInt.prototype.toJSON !== "function") {
  // @ts-ignore
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}
import { Call, TokenOrder, Ucs03, Ucs05, Utils, ZkgmInstruction } from "@unionlabs/sdk"
import { Cosmos } from "@unionlabs/sdk-cosmos"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import * as Token from "@unionlabs/sdk/Token"
import { Console } from "effect"
import * as A from "effect/Array"
import * as Cause from "effect/Cause"
import * as Effect from "effect/Effect"
import { encodeBase64 } from "effect/Encoding"
import { pipe } from "effect/Function"
import * as Match from "effect/Match"
import * as ParseResult from "effect/ParseResult"
import * as Schema from "effect/Schema"
import { bytesToHex, fromHex, keccak256 } from "viem"

const OSMOSIS_CHAIN_ID = UniversalChainId.make("osmosis.osmosis-1")
const ETHEREUM_CHAIN_ID = UniversalChainId.make("ethereum.1")

const ATOMONE_TO_OSMOSIS_CHANNEL = "channel-2"
const ATOMONE_TO_OSMOSIS_PORT = "transfer"

const ZKGM_ADDRESS = "osmo1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qs2uecc"

const SOURCE_CHANNEL_ID = ChannelId.make(2)

export const ATONE_SOLVER_ON_OSMOSIS_METADATA =
  "000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000003f6f736d6f316174306e6539617977683335706d6c7a3065786c35666c336c6a7770657934676336323079797874687173706477396536306d736c666a786d33000000000000000000000000000000000000000000000000000000000000000000" as const

export const ATONE_SOLVER_ON_ETH_METADATA =
  "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014a1a1d0b9182339e86e80db519218ea03ec09a1a10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" as const

export const ATONE_IBC_DENOM_ON_OSMOSIS = Token.CosmosIbcClassic.make({
  address: "ibc/BC26A7A805ECD6822719472BCB7842A48EF09DF206182F8F259B2593EB5D23FB",
})

export const ATONE_ERC20 = Token.Erc20.make({
  address: "0xA1a1d0B9182339e86e80db519218eA03Ec09a1A1",
})

const PROXY_ACCOUNT_FACTORY = Ucs05.CosmosDisplay.make({
  address: "osmo13jcvgpy2cjl6tg7zz5pcr9pv6lgqz70h7n64krjve7mp7tsexvys82mlqs",
})

const SEND_AMOUNT = 1n
const SENDER_ATOMONE = Ucs05.CosmosDisplay.make({
  address: "atone19lnpcs0pvz9htcvm58jkp6ak55m49x5nr0w9qj",
})
const REFUND_RECEIVER_OSMOSIS = Ucs05.CosmosDisplay.make({
  address: Schema.decodeUnknownSync(
    Ucs05.Bech32FromCanonicalBytesWithPrefix(
      "osmo",
    ),
  )(
    Ucs05.anyDisplayToCanonical(SENDER_ATOMONE),
  ),
})
const RECEIVER = Ucs05.EvmDisplay.make({
  address: "0x2c96e52fce14baa13868ca8182f8a7903e4e76e0",
})

const sha256 = (data: BufferSource) =>
  Effect.tryPromise(() => globalThis.crypto.subtle.digest("SHA-256", data))

const calculateIbcCallbackAddress = Effect.fn("calculateIbcCallbackAddress")(
  function*(sender: string, channelId: string) {
    const preimage = new Uint8Array([
      ...new Uint8Array(
        yield* sha256(new globalThis.TextEncoder().encode("ibc-wasm-hook-intermediary")),
      ),
      ...new globalThis.TextEncoder().encode(`${channelId}/${sender}`),
    ])

    const addr = Ucs05.CosmosDisplay.make({
      address: yield* Schema.decode(
        Ucs05.Bech32FromCanonicalBytesWithPrefix(
          "osmo",
        ),
      )(
        `0x${yield* Schema.encode(Schema.Uint8ArrayFromHex)(
          new Uint8Array(yield* sha256(preimage)),
        )}`,
      ),
    })

    yield* Console.log({ addr, preimage })

    return addr
  },
)

// proxy_account_factory::predict_call_proxy_account
export const predictProxy = Effect.fn("predictProxy")(
  function*(sender: Ucs05.CosmosDisplay) {
    const BYTECODE_BASE_CHECKSUM =
      "0xec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1" as const
    const CANONICAL_PROXY_ACCOUNT_FACTORY = Ucs05.anyDisplayToCanonical(
      PROXY_ACCOUNT_FACTORY,
    )
    // shaw256(b"module")
    const MODULE_HASH =
      "0x120970d812836f19888625587a4606a5ad23cef31c8684e601771552548fc6b9" as const

    const canonical_sender = fromHex(Ucs05.anyDisplayToCanonical(sender), "bytes")
    const salt = yield* sha256(canonical_sender.buffer)

    const u64toBeBytes = (n: bigint) => {
      const buffer = new ArrayBuffer(8)
      const view = new DataView(buffer)
      view.setBigUint64(0, n)
      return new Uint8Array(view.buffer)
    }

    const address = yield* pipe(
      Uint8Array.from(
        [
          ...fromHex(MODULE_HASH, "bytes"),
          ...new TextEncoder().encode("wasm"),
          0,
          ...u64toBeBytes(32n),
          ...fromHex(BYTECODE_BASE_CHECKSUM, "bytes"),
          ...u64toBeBytes(32n),
          ...fromHex(CANONICAL_PROXY_ACCOUNT_FACTORY, "bytes"),
          ...u64toBeBytes(32n),
          ...new Uint8Array(salt),
          ...u64toBeBytes(0n),
        ],
      ),
      sha256,
      Effect.map((r) => new Uint8Array(r)),
      Effect.map(bytesToHex),
      Effect.flatMap(
        Schema.decode(Ucs05.Bech32FromCanonicalBytesWithPrefix("osmo")),
      ),
    )

    yield* Console.log({ address })

    return Ucs05.CosmosDisplay.make({ address })
  },
)

const encodeInstruction: (
  u: ZkgmInstruction.ZkgmInstruction,
) => Effect.Effect<
  Ucs03.Ucs03,
  ParseResult.ParseError | Cause.TimeoutException | Cosmos.QueryContractError
> = pipe(
  Match.type<ZkgmInstruction.ZkgmInstruction>(),
  Match.tagsExhaustive({
    Batch: (batch) =>
      pipe(
        batch.instructions,
        A.map(encodeInstruction),
        Effect.allWith({ concurrency: "unbounded" }),
        Effect.map((operand) =>
          new Ucs03.Batch({
            opcode: batch.opcode,
            version: batch.version,
            operand,
          })
        ),
      ),
    TokenOrder: TokenOrder.encodeV2,
    Call: Call.encode,
  }),
)

const encodeUcs03 = (payload: {
  instruction: ZkgmInstruction.ZkgmInstruction
  sourceChannelId: number
}) =>
  Effect.gen(function*() {
    const salt = yield* Utils.generateSalt("cosmos")
    const timeout_timestamp = "1"
    const instruction = yield* pipe(
      encodeInstruction(payload.instruction), //
      Effect.flatMap(Schema.encode(Ucs03.Ucs03WithInstructionFromHex)),
    )
    return {
      send: {
        channel_id: ChannelId.make(payload.sourceChannelId),
        timeout_height: "0",
        timeout_timestamp,
        salt,
        instruction,
      },
    }
  }).pipe(
    Effect.runPromise,
  )

const createUcs03 = Effect.gen(function*() {
  const osmosisChain = yield* ChainRegistry.byUniversalId(OSMOSIS_CHAIN_ID)
  const ethereumChain = yield* ChainRegistry.byUniversalId(ETHEREUM_CHAIN_ID)

  return yield* TokenOrder.make({
    source: ethereumChain,
    destination: osmosisChain,
    // sender: yield* predictProxy(
    //   yield* calculateIbcCallbackAddress(SENDER_ATOMONE.address, ATOMONE_TO_OSMOSIS_CHANNEL),
    // ),
    sender: REFUND_RECEIVER_OSMOSIS,
    receiver: RECEIVER,
    baseToken: ATONE_IBC_DENOM_ON_OSMOSIS,
    baseAmount: SEND_AMOUNT,
    quoteToken: ATONE_ERC20,
    quoteAmount: SEND_AMOUNT,
    kind: "solve",
    metadata: ATONE_SOLVER_ON_ETH_METADATA,
    version: 2,
  })
}).pipe(
  Effect.provide(ChainRegistry.Default),
  Effect.runPromise,
)

createUcs03.then(
  (instruction) =>
    encodeUcs03({
      instruction,
      sourceChannelId: SOURCE_CHANNEL_ID,
    }),
)
  .then(msg =>
    JSON.stringify({
      // wasm: {
      //   contract: PROXY_ACCOUNT_FACTORY.address,
      //   msg: {
      //     call_proxy: [
      //       {
      //         wasm: {
      //           execute: {
      //             contract_addr: ZKGM_ADDRESS,
      //             msg: encodeBase64(JSON.stringify(msg)),
      //             funds: [
      //               {
      //                 amount: SEND_AMOUNT.toString(),
      //                 denom: ATONE_IBC_DENOM_ON_OSMOSIS.address,
      //               },
      //             ],
      //           },
      //         },
      //       },
      //     ],
      //   },
      // },
      wasm: {
        contract: ZKGM_ADDRESS,
        msg,
      },
    })
  )
  .then(console.log)
  .then(console.error)
