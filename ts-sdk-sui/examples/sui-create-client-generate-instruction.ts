// @ts-ignore
if (typeof BigInt.prototype.toJSON !== "function") {
  // @ts-ignore
  BigInt.prototype.toJSON = function () {
    return this.toString()
  }
}
import { Effect, Logger } from "effect"
import { getFullnodeUrl } from "@mysten/sui/client"
import { PublicClient, WalletClient, writeContract, readCoinMetadata, readCoinBalances, sendInstruction } from "../src/Sui.js"
import { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"
import * as ZkgmClientRequest from "@unionlabs/sdk/ZkgmClientRequest"
import * as ZkgmClientResponse from "@unionlabs/sdk/ZkgmClientResponse"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import * as ZkgmIncomingMessage from "@unionlabs/sdk/ZkgmIncomingMessage"
import * as ZkgmClient from "@unionlabs/sdk/ZkgmClient"
import { layerWithoutWallet } from "../src/SuiZkgmClient.js"

import { Transaction } from "@mysten/sui/transactions"
import * as TokenOrder from "@unionlabs/sdk/TokenOrder"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"

const MNEMONIC = process.env.SUI_MNEMONIC ?? "..."
const RECIPIENT = process.env.RECIPIENT ?? "0x03ff9dd9e093387bdd4432c6a3eb6a1bd5a8f39a530042ac7efe576f18d3232b"

const keypair = Ed25519Keypair.deriveKeypair(MNEMONIC)


const program = Effect.gen(function* () {

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

  const sender = wallet.signer.toSuiAddress();


  // deployed contract: union1l0rpy8yauy7nzv4vu6mgz6kjpqzvws85l8mgzm6eansasx90t57sc7k4ue

  console.log("sender:", sender)

  const tokenOrder = yield* TokenOrder.make({
    source,
    destination,
    sender: "0x06627714f3F17a701f7074a12C02847a5D2Ca487",
    receiver: "0x756E696F6E317779637938673876357366663667736A6C3979686A73343371393878706C30357033676E3273",
    baseToken: "0x3078323A3A7375693A3A535549",
    baseAmount: 100000n,
    quoteToken: "0x756E696F6E316C307270793879617579376E7A76347675366D677A366B6A70717A76777338356C386D677A6D3665616E7361737839307435377363376B347565",
    quoteAmount: 10000n,
    kind: "solve",
    metadata:
      "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040756e696f6e31736b673532343468706b61643630337a7a37376b64656b7a77366666677066726465336c646b387270647a30366e36326b34687163743077346a",
    version: 2,
  })

  const request = ZkgmClientRequest.make({
    source,
    destination,
    channelId: ChannelId.make(1),
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
