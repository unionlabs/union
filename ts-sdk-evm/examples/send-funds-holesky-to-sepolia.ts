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
import { TokenOrder, ZkgmClient, ZkgmClientRequest, ZkgmClientResponse } from "@unionlabs/sdk"
import { Evm, EvmZkgmClient } from "@unionlabs/sdk-evm"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
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
    baseAmount: 101n,
    quoteToken: "0x80fdbf104ec58a527ec40f7b03f88c404ef4ba63",
    quoteAmount: 101n,
    kind: TokenOrder.Kind.Escrow,
    metadata: undefined,
  })

  // const batch = tokenOrder.pipe(
  //   Effect.map(TokenOrder.withFee({ priority: "high" }))
  // )

  console.log({ tokenOrder })

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

  // const completion = yield* response.waitFor(ZkgmIncomingMessage.isComplete)
  // NOTE: 4. tx complete

  yield* Effect.log("TX Hash:", response.txHash)
}).pipe(
  Effect.provide(EvmZkgmClient.layerWithoutWallet),
  Effect.provide(Evm.WalletClient.Live({
    account: privateKeyToAccount(
      "0x...",
    ),
    chain: holesky,
    transport: http("https://rpc.17000.holesky.chain.kitchen"),
  })),
  Effect.provide(Evm.PublicClient.Live({
    chain: holesky,
    transport: http("https://rpc.17000.holesky.chain.kitchen"),
  })),
  // Effect.provide(ChannelRegistry.Default),
  // Effect.provide(FeeEstimator.Default),
  // Effect.provide(TokenRegistry.Default),
  Effect.provide(ChainRegistry.Default),
  Effect.provide(Logger.replace(Logger.defaultLogger, Logger.prettyLoggerDefault)),
)

Effect.runPromise(program as unknown as any)
  .then(console.log)
  .catch(console.error)
