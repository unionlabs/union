/**
 * @title Send Funds Holesky → Sepolia
 * @description Example transfer from Holesky to Sepolia.
 * @badge ✓:success
 */
/// <reference types="effect" />
/// <reference types="viem" />
// @paths: {"@unionlabs/sdk": ["../ts-sdk/src"], "@unionlabs/sdk/*": ["../ts-sdk/src/*"]}
// @paths: {"@unionlabs/sdk-evm": ["../ts-sdk-evm/src"], "@unionlabs/sdk-evm/*": ["../ts-sdk-evm/src/*"]}
// @ts-ignore
if (typeof BigInt.prototype.toJSON !== "function") {
  // @ts-ignore
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}
// ---cut---
import { Evm, EvmZkgmClient } from "@unionlabs/sdk-evm"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import * as TokenOrder from "@unionlabs/sdk/TokenOrder"
import * as ZkgmClient from "@unionlabs/sdk/ZkgmClient"
import * as ZkgmClientRequest from "@unionlabs/sdk/ZkgmClientRequest"
import * as ZkgmClientResponse from "@unionlabs/sdk/ZkgmClientResponse"
import * as ZkgmIncomingMessage from "@unionlabs/sdk/ZkgmIncomingMessage"
import { Effect, Logger } from "effect"
import { http } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { holesky } from "viem/chains"

const program = Effect.gen(function*() {
  const source = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("ethereum.17000"),
  )
  const destination = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("ethereum.11155111"),
  )

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
  Effect.provide(EvmZkgmClient.layerWithoutWallet),
  Effect.provide(Evm.WalletClient.Live({
    account: privateKeyToAccount(
      (process.env.KEY as any) ?? "0x...",
    ),
    chain: holesky,
    transport: http("https://rpc.17000.ethereum.chain.kitchen"),
  })),
  Effect.provide(Evm.PublicClient.Live({
    chain: holesky,
    transport: http("https://rpc.17000.ethereum.chain.kitchen"),
  })),
  Effect.provide(ChainRegistry.Default),
  Effect.provide(Logger.replace(Logger.defaultLogger, Logger.prettyLoggerDefault)),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
