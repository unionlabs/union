/**
 * @title Send Funds Holesky → Sepolia
 * @description Example transfer from Holesky to Sepolia.
 * @badge ✓:success
 */
/// <reference types="effect" />
/// <reference types="viem" />
// @paths: {"@unionlabs/sdk": ["../ts-sdk/src"], "@unionlabs/sdk/*": ["../ts-sdk/src/*"]}
// @paths: {"@unionlabs/sdk-cosmos": ["../ts-sdk-cosmos/src"], "@unionlabs/sdk-cosmos/*": ["../ts-sdk-cosmos/src/*"]}
// @ts-ignore
if (typeof BigInt.prototype.toJSON !== "function") {
  // @ts-ignore
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}
import { Decimal } from "@cosmjs/math"
// ---cut---
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { GasPrice } from "@cosmjs/stargate"
import { TokenOrder, ZkgmClient, ZkgmClientRequest, ZkgmClientResponse } from "@unionlabs/sdk"
import { Cosmos, CosmosZkgmClient } from "@unionlabs/sdk-cosmos"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { Effect, Logger } from "effect"

const program = Effect.gen(function*() {
  const source = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("union.union-testnet-10"),
  )
  const destination = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("ethereum.11155111"),
  )

  const tokenOrder = yield* TokenOrder.make({
    source,
    destination,
    sender: "union122ny3mep2l7nhtafpwav2y9e5jrslhek76hsjl",
    receiver: "0x50A22f95bcB21E7bFb63c7A8544AC0683dCeA302",
    baseToken: "au",
    baseAmount: 10n,
    quoteToken: "0xba5eD44733953d79717F6269357C77718C8Ba5ed",
    quoteAmount: 10n,
    kind: "solve",
    metadata:
      "0x000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000014ba5ed44733953d79717f6269357c77718c8ba5ed0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    version: 2,
  })

  const request = ZkgmClientRequest.make({
    source,
    destination,
    channelId: ChannelId.make(1),
    ucs03Address: "union1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qpe64fh",
    instruction: tokenOrder,
  })

  const zkgmClient = yield* ZkgmClient.ZkgmClient

  const response: ZkgmClientResponse.ZkgmClientResponse = yield* zkgmClient.execute(request)

  yield* Effect.log("TX Hash:", response.txHash)
}).pipe(
  Effect.provide(CosmosZkgmClient.layerWithoutSigningClient),
  Effect.provide(Cosmos.SigningClient.Live(
    "union122ny3mep2l7nhtafpwav2y9e5jrslhek76hsjl",
    "https://rpc.union-testnet-10.union.chain.kitchen",
    await DirectSecp256k1HdWallet.fromMnemonic(
      "memo memo memo",
      { prefix: "union" },
    ),
    { gasPrice: new GasPrice(Decimal.one(18), "au") },
  )),
  Effect.provide(Cosmos.Client.Live("https://rpc.union-testnet-10.union.chain.kitchen")),
  Effect.provide(ChainRegistry.Default),
  Effect.provide(Logger.replace(Logger.defaultLogger, Logger.prettyLoggerDefault)),
)

Effect.runPromise(program as unknown as any)
  .then(console.log)
  .catch(console.error)
