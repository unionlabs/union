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
    UniversalChainId.make("ethereum.11155111"),
  )
  const wallet = yield* WalletClient

  const sender = wallet.signer.toSuiAddress();

  // TODO: Fix this tokenOrder and write something working
  const tokenOrder = yield* TokenOrder.make({
    source,
    destination,
    sender: "0x06627714f3F17a701f7074a12C02847a5D2Ca487",
    receiver: "0x50A22f95bcB21E7bFb63c7A8544AC0683dCeA302",
    // LINK on Holesky
    baseToken: "0x685ce6742351ae9b618f383883d6d1e0c5a31b4b",
    baseAmount: 10n,
    // Holesky LINK on Sepolia
    quoteToken: "0x80fdbf104ec58a527ec40f7b03f88c404ef4ba63",
    quoteAmount: 10n,
    kind: "escrow",
    metadata: undefined,
    version: 2,
  })

  yield* Effect.log("Token Order V2", tokenOrder)

  // TODO: Fix this request too
  const request = ZkgmClientRequest.make({
    source,
    destination,
    channelId: ChannelId.make(1),
    ucs03Address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
    kind: "simulateAndExecute",
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
    signer: keypair,               // ✅ Sui signer
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
