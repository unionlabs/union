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
import * as ZkgmIncomingMessage from "@unionlabs/sdk/ZkgmIncomingMessage"
import { Effect, Logger } from "effect"
import {
  PublicClient,
  readCoinBalances,
  readCoinMetadata,
  sendInstruction,
  WalletClient,
  writeContract,
} from "../src/Sui.js"
import { layerWithoutWallet } from "../src/SuiZkgmClient.js"

import { Transaction } from "@mysten/sui/transactions"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import * as TokenOrder from "@unionlabs/sdk/TokenOrder"

const MNEMONIC = process.env.SUI_MNEMONIC ?? "..."
const RECIPIENT = process.env.RECIPIENT
  ?? "0x03ff9dd9e093387bdd4432c6a3eb6a1bd5a8f39a530042ac7efe576f18d3232b"

const keypair = Ed25519Keypair.deriveKeypair(MNEMONIC)

const program = Effect.gen(function*() {
  // TODO: Source will be SUI testnet
  const source = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("ethereum.17000"),
  )

  console.log("source", source)

  // TODO: Destination will be somewhere
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
    receiver: "union1wycy8g8v5sff6gsjl9yhjs43q98xpl05p3gn2s",
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
    ucs03Address: "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c",
    instruction: tokenOrder,
  })

  const zkgmClient = yield* ZkgmClient.ZkgmClient

  const response: ZkgmClientResponse.ZkgmClientResponse = yield* zkgmClient.execute(request)

  yield* Effect.log("Submission Hash", response.txHash)

  const completion = yield* response.waitFor(
    ZkgmIncomingMessage.LifecycleEvent.$is("EvmTransactionReceiptComplete"),
  )

  yield* Effect.log("Completion", completion)
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
