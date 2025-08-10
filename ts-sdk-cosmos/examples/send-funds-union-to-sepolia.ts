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
// ---cut---
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { GasPrice } from "@cosmjs/stargate"
import { TokenOrder, ZkgmClient, ZkgmClientRequest, ZkgmClientResponse } from "@unionlabs/sdk"
import { Cosmos, CosmosZkgmClient } from "@unionlabs/sdk-cosmos"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { Effect, Logger } from "effect"

// has a function .encode() -> ethabi (uses Ucs03 module)
// has a function .extractRequiredTokens() -> Token[]
//                                           example output: [{ token: Token.Erc20(`0x1234`), amount: 42342n }, { token: Token.EvmGas, amount: 200n }]

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
    sender: "union122ny3mep2l7nhtafpwav2y9e5jrslhek76hsjl",
    receiver: "0x50A22f95bcB21E7bFb63c7A8544AC0683dCeA302",
    // LINK on Holesky
    baseToken: "0x685ce6742351ae9b618f383883d6d1e0c5a31b4b",
    baseAmount: 100n,
    quoteToken: "0x80fdbf104ec58a527ec40f7b03f88c404ef4ba63",
    quoteAmount: 100n,
    kind: TokenOrder.Kind.Escrow,
    metadata: undefined,
  })

  const request = ZkgmClientRequest.make({
    source,
    destination,
    channelId: ChannelId.make(2),
    ucs03Address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
    instruction: tokenOrder,
  })

  const zkgmClient = yield* ZkgmClient.ZkgmClient

  const response: ZkgmClientResponse.ZkgmClientResponse = yield* zkgmClient.execute(request)

  // const completion = yield* response.waitFor(ZkgmIncomingMessage.isComplete)

  yield* Effect.log("TX Hash:", response.txHash)
}).pipe(
  Effect.provide(CosmosZkgmClient.layerWithoutSigningClient),
  Effect.provide(Cosmos.SigningClient.Live(
    "union122ny3mep2l7nhtafpwav2y9e5jrslhek76hsjl",
    "https://rpc.union-testnet-10.union.chain.kitchen",
    await DirectSecp256k1HdWallet.fromMnemonic("memo memo memo", { prefix: "union" }),
    { gasPrice: GasPrice.fromString("0.007ubbn") },
  )),
  Effect.provide(Cosmos.Client.Live("https://rpc.union-testnet-10.union.chain.kitchen")),
  Effect.provide(ChainRegistry.Default),
  Effect.provide(Logger.replace(Logger.defaultLogger, Logger.prettyLoggerDefault)),
)

Effect.runPromise(program as unknown as any)
  .then(console.log)
  .catch(console.error)
