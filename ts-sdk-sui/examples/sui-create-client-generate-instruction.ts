// @ts-ignore
if (typeof BigInt.prototype.toJSON !== "function") {
  // @ts-ignore
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}
import { getFullnodeUrl } from "@mysten/sui/client"
import { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import * as ZkgmClient from "@unionlabs/sdk/ZkgmClient"
import * as ZkgmClientRequest from "@unionlabs/sdk/ZkgmClientRequest"
import * as ZkgmClientResponse from "@unionlabs/sdk/ZkgmClientResponse"
import { Effect, Logger } from "effect"
import { PublicClient, WalletClient } from "../src/Sui.js"
import { layerWithoutWallet } from "../src/SuiZkgmClient.js"

import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import * as TokenOrder from "@unionlabs/sdk/TokenOrder"

const MNEMONIC = process.env.SUI_MNEMONIC ?? "..."
const RECIPIENT = process.env.RECIPIENT
  ?? "union1wycy8g8v5sff6gsjl9yhjs43q98xpl05p3gn2s"

const keypair = Ed25519Keypair.deriveKeypair(MNEMONIC)

const program = Effect.gen(function*() {
  // TODO: Source will be SUI testnet
  const source = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("ethereum.17000"),
  )

  console.log("source", source)

  const destination = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("union.union-1"),
  )
  const wallet = yield* WalletClient

  const sender = wallet.signer.toSuiAddress()

  console.log("sender:", sender)

  const tokenOrder = yield* TokenOrder.make({
    source,
    destination,
    sender: sender,
    receiver: RECIPIENT,
    baseToken: "0x2::sui::SUI",
    baseAmount: 10000000n,
    quoteToken: "union1y05e0p2jcvhjzf7kcqsrqx93d4g3u93hc2hykaq8hrvkqrp5ltrssagzyd",
    quoteAmount: 10000000n,
    kind: "solve",
    metadata:
      "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040756e696f6e31793035653070326a6376686a7a66376b63717372717839336434673375393368633268796b6171386872766b717270356c7472737361677a7964",
    version: 2,
  })

  const request = ZkgmClientRequest.make({
    source,
    destination,
    channelId: ChannelId.make(5),
    ucs03Address:
      "0x3078623965306634373861623162623735393639343336386465363263656437636230343035376634383964353666326230613661376435656263313430373037393a3a7a6b676d3a3a307861316362663135656236333166303139323234643530613935373035356464663931313331333636303133323934393438656535346537663036666635383462",
    instruction: tokenOrder,

    // NEW — only read by the Sui client
    transport: {
      sui: {
        vaultId: "0x7c4ade19208295ed6bf3c4b58487aa4b917ba87d31460e9e7a917f7f12207ca3",
        ibcStoreId: "0xac7814eebdfbf975235bbb796e07533718a9d83201346769e5f281dc90009175",
        coins: [
          {
            typeArg: "0x2::sui::SUI",
            objectId: "0x266d00c4b329111255339c041cc57a1b616cfeddafdae47df8f814002578e95b",
            baseAmount: BigInt(3),
          },
        ],
      },
    },
  })

  yield* Effect.log("ZKGM Client Request", request)

  const zkgmClient = yield* ZkgmClient.ZkgmClient

  const response: ZkgmClientResponse.ZkgmClientResponse = yield* zkgmClient.execute(request)

  yield* Effect.log("Submission Hash", response.txHash)
}).pipe(
  Effect.provide(layerWithoutWallet),
  Effect.provide(PublicClient.Live({ url: getFullnodeUrl("testnet") })),
  Effect.provide(
    WalletClient.Live({
      url: getFullnodeUrl("testnet"),
      signer: keypair,
    }),
  ),
  Effect.provide(ChainRegistry.Default),
  Effect.provide(Logger.replace(Logger.defaultLogger, Logger.prettyLoggerDefault)),
)

Effect.runPromise(program).catch((e: any) => {
  console.error("\n--- TOP-LEVEL ERROR ---")
  console.dir(e, { depth: 10 })
  if (e?.cause) {
    console.error("\n--- ORIGINAL CAUSE ---")
    console.dir(e.cause, { depth: 10 })
  }
})
