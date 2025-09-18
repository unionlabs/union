/**
 * @title Call
 * @description Example call on Babylon to Ethereum contract.
 * @badge ✓:success
 */
/// <reference types="effect" />
/// <reference types="@cosmjs/math" />
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
import { Call, Ucs05, ZkgmClientRequest, ZkgmClientResponse } from "@unionlabs/sdk"
import { Cosmos, CosmosZkgmClient } from "@unionlabs/sdk-cosmos"
import { ChainRegistry } from "@unionlabs/sdk/ChainRegistry"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { Effect, Logger } from "effect"

const signer = await DirectSecp256k1HdWallet.fromMnemonic(
  process.env.MEMO ?? "memo memo memo",
  { prefix: "bbn" },
)

const program = Effect.gen(function*() {
  const source = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("babylon.bbn-1"),
  )

  const destination = yield* ChainRegistry.byUniversalId(
    UniversalChainId.make("ethereum.1"),
  )

  const call = Call.make({
    sender: Ucs05.CosmosDisplay.make({
      address: "bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh",
    }),
    eureka: false,
    contractAddress: Ucs05.EvmDisplay.make({
      address: "0x921e5b5091f431f84f14423ec487783a853bc4b0",
    }),
    contractCalldata: "0xDEADBEEF",
  })

  const request = ZkgmClientRequest.make({
    source,
    destination,
    channelId: ChannelId.make(3),
    ucs03Address: "bbn1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292q77945h",
    instruction: call,
  })

  const client = yield* CosmosZkgmClient.make.pipe(
    Effect.provide(Cosmos.SigningClient.Live(
      "bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh",
      "https://rpc.bbn-1.babylon.chain.kitchen",
      signer,
      { gasPrice: GasPrice.fromString("0.0007ubbn") },
    )),
    Effect.provide(Cosmos.Client.Live("https://rpc.bbn-1.babylon.chain.kitchen")),
  )

  const response: ZkgmClientResponse.ZkgmClientResponse = yield* client.execute(request)

  yield* Effect.log("TX Hash:", response.txHash)
}).pipe(
  Effect.provide(ChainRegistry.Default),
  Effect.provide(Logger.replace(Logger.defaultLogger, Logger.prettyLoggerDefault)),
)

Effect.runPromise(program)
  .then(console.log)
  .catch(console.error)
