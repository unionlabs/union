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

const MNEMONIC = process.env.MNEMONIC ?? "fix auto gallery heart practice drip joke nice decline lift attend bread"
const RECIPIENT = process.env.RECIPIENT ?? "0x03ff9dd9e093387bdd4432c6a3eb6a1bd5a8f39a530042ac7efe576f18d3232b"

const keypair = Ed25519Keypair.deriveKeypair(MNEMONIC)


const program = Effect.gen(function* () {
  const source = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("ethereum.17000"),
  )

  console.log("source", source)
  const destination = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("ethereum.11155111"),
  )
  const { client } = yield* PublicClient

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

  const request = ZkgmClientRequest.make({
    source,
    destination,
    channelId: ChannelId.make(2),
    ucs03Address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
    kind: "simulateAndExecute",
    instruction: tokenOrder,
  })

  const zkgmClient = yield* ZkgmClient.ZkgmClient

  // NOTE: 1. switch chain is assumed
  // NOTE: 2. write in progress

  const response: ZkgmClientResponse.ZkgmClientResponse = yield* zkgmClient.execute(request)

  // NOTE: 3. write complete (with tx hash)

  yield* Effect.log("Submission Hash", response.txHash)

  const completion = yield* response.waitFor(
    ZkgmIncomingMessage.LifecycleEvent.$is("EvmTransactionReceiptComplete"),
  )

  // NOTE: 4. tx complete

  yield* Effect.log("Completion", completion)

  
}).pipe(
  Effect.provide(layerWithoutWallet),
  Effect.provide(PublicClient.Live({ url: getFullnodeUrl("testnet") })),
  Effect.provide(
    WalletClient.Live({
      url: getFullnodeUrl("testnet"),
      account: keypair,              // signer
      chain: "sui-testnet" as any,   // placeholder; not used internally
    }),
  ),

  Effect.provide(ChainRegistry.Default),
  Effect.provide(Logger.replace(Logger.defaultLogger, Logger.prettyLoggerDefault)),
)

Effect.runPromise(program).catch(console.error)
