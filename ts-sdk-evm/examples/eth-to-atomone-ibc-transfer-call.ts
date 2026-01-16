// @ts-ignore
if (typeof BigInt.prototype.toJSON !== "function") {
  // @ts-ignore
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}
import {
  Batch,
  Call,
  TokenOrder,
  Ucs03,
  Ucs05,
  Utils,
  ZkgmClient,
  ZkgmClientRequest,
  ZkgmInstruction,
} from "@unionlabs/sdk"
import { Evm, EvmZkgmClient } from "@unionlabs/sdk-evm"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { HexFromJson } from "@unionlabs/sdk/schema/hex"
import * as Token from "@unionlabs/sdk/Token"
import * as A from "effect/Array"
import * as Cause from "effect/Cause"
import * as Effect from "effect/Effect"
import { pipe } from "effect/Function"
import * as Match from "effect/Match"
import * as ParseResult from "effect/ParseResult"
import * as Schema from "effect/Schema"
import { bytesToHex, encodeAbiParameters, fromHex, http, keccak256 } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { mainnet } from "viem/chains"

const OSMOSIS_CHAIN_ID = UniversalChainId.make("osmosis.osmosis-1")
const ETHEREUM_CHAIN_ID = UniversalChainId.make("ethereum.1")

const OSMOSIS_TO_ATOMONE_CHANNEL = "channel-94814"

const UCS03_EVM = Ucs05.EvmDisplay.make({ address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03" })

const SOURCE_CHANNEL_ID = ChannelId.make(6)
const DESTINATION_CHANNEL_ID = ChannelId.make(2)
const SEND_AMOUNT = 1n
const RECEIVER_ATOMONE = Ucs05.CosmosDisplay.make({
  address: "atone19lnpcs0pvz9htcvm58jkp6ak55m49x5nr0w9qj",
})
const SENDER_ETH = Ucs05.EvmDisplay.make({
  address: "0x2c96e52fce14baa13868ca8182f8a7903e4e76e0",
})

const BYTECODE_BASE_CHECKSUM =
  "0xec827349ed4c1fec5a9c3462ff7c979d4c40e7aa43b16ed34469d04ff835f2a1" as const
// shaw256(b"module")
const MODULE_HASH = "0x120970d812836f19888625587a4606a5ad23cef31c8684e601771552548fc6b9" as const
const UCS03_ZKGM = Ucs05.CosmosDisplay.make({
  address: "osmo1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qs2uecc",
})
const CANONICAL_ZKGM = Ucs05.anyDisplayToCanonical(UCS03_ZKGM)

export const ATONE_SOLVER_ON_OSMOSIS_METADATA =
  "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000003f6f736d6f316174306e6539617977683335706d6c7a3065786c35666c336c6a7770657934676336323079797874687173706477396536306d736c666a786d33000000000000000000000000000000000000000000000000000000000000000000" as const

export const ATONE_SOLVER_ON_ETH_METADATA =
  "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014a1a1d0b9182339e86e80db519218ea03ec09a1a10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" as const

export const ATONE_IBC_DENOM_ON_OSMOSIS = Token.CosmosIbcClassic.make({
  address: "ibc/BC26A7A805ECD6822719472BCB7842A48EF09DF206182F8F259B2593EB5D23FB",
})

export const ATONE_ERC20 = Token.Erc20.make({
  address: "0xA1a1d0B9182339e86e80db519218eA03Ec09a1A1",
})

/**
 * Generate a deterministic Union cosmos address from an EVM address using instantiate2
 * This is used to create the receiver address for cross-chain operations
 */
export const predictProxy = Effect.fn("predictProxy")(
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
          ...fromHex(MODULE_HASH, "bytes"),
          ...new TextEncoder().encode("wasm"),
          0,
          ...u64toBeBytes(32n),
          ...fromHex(BYTECODE_BASE_CHECKSUM, "bytes"),
          ...u64toBeBytes(32n),
          ...fromHex(CANONICAL_ZKGM, "bytes"),
          ...u64toBeBytes(32n),
          ...salt,
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

    return Ucs05.CosmosDisplay.make({ address })
  },
)

Effect.gen(function*() {
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

  const osmosisChain = yield* ChainRegistry.byUniversalId(OSMOSIS_CHAIN_ID)
  const ethereumChain = yield* ChainRegistry.byUniversalId(ETHEREUM_CHAIN_ID)

  const proxy = yield* predictProxy({
    path: 0n,
    channel: DESTINATION_CHANNEL_ID,
    sender: SENDER_ETH,
  })

  const timeout_timestamp = Utils.getTimeoutInNanoseconds24HoursFromNow()

  const tokenOrder = yield* TokenOrder.make({
    source: ethereumChain,
    destination: osmosisChain,
    sender: SENDER_ETH,
    receiver: proxy,
    baseToken: ATONE_ERC20,
    baseAmount: SEND_AMOUNT,
    quoteToken: ATONE_IBC_DENOM_ON_OSMOSIS,
    quoteAmount: SEND_AMOUNT,
    kind: "solve",
    metadata: ATONE_SOLVER_ON_OSMOSIS_METADATA,
    version: 2,
  })

  const ibcTransferCall = {
    ibc: {
      transfer: {
        channel_id: OSMOSIS_TO_ATOMONE_CHANNEL,
        to_address: RECEIVER_ATOMONE.address,
        amount: {
          denom: ATONE_IBC_DENOM_ON_OSMOSIS.address,
          amount: SEND_AMOUNT,
        },
        timeout: {
          timestamp: timeout_timestamp,
        },
        memo: "",
      },
    },
  }

  const calls = Call.make({
    sender: SENDER_ETH,
    eureka: false,
    contractAddress: proxy,
    contractCalldata: yield* Schema.decode(HexFromJson)([
      ibcTransferCall,
    ]),
  })

  const batch = Batch.make([
    tokenOrder,
    calls,
  ])

  const request = ZkgmClientRequest.make({
    source: ethereumChain,
    destination: osmosisChain,
    channelId: SOURCE_CHANNEL_ID,
    ucs03Address: UCS03_EVM.address,
    instruction: batch,
  })

  const client = yield* ZkgmClient.ZkgmClient
  return yield* client.execute(request)
}).pipe(
  Effect.provide(ChainRegistry.Default),
  Effect.provide(EvmZkgmClient.layerWithoutWallet),
  Effect.provide(Evm.WalletClient.Live({
    account: privateKeyToAccount(
      (process.env.KEY as any) ?? "0x...",
    ),
    chain: mainnet,
    transport: http("https://rpc.1.ethereum.chain.kitchen"),
  })),
  Effect.provide(Evm.PublicClient.Live({
    chain: mainnet,
    transport: http("https://rpc.1.ethereum.chain.kitchen"),
  })),
  Effect.runPromise,
)
  .then(console.log)
  .then(console.error)
