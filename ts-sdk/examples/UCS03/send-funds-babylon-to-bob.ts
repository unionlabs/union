/**
 * @title Send Funds Babylon → BOB
 * @description Example transfer from Babylon to BOB.
 * @badge ✓:success
 */
/// <reference types="effect" />
/// <reference types="viem" />
/// <reference types="@cosmjs/proto-signing" />
/// <reference types="@cosmjs/stargate" />
// @paths: {"@unionlabs/sdk": ["../ts-sdk/src"], "@unionlabs/sdk/*": ["../ts-sdk/src/*"]}

// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}
// ---cut---
// CosmWasm
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { GasPrice } from "@cosmjs/stargate"
// EVM
import { http, toHex } from "viem"
import { bob } from "viem/chains"
// Union
import { Cosmos, Evm, FungibleAssetOrder, Ucs03, Ucs05 } from "@unionlabs/sdk"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { Effect, pipe } from "effect"

// We will send funds from sender to receiver
const SENDER = Ucs05.AddressCosmosZkgm.make(toHex("bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh"))
const RECEIVER = Ucs05.AddressEvmZkgm.make("0xfaebe5bf141cc04a3f0598062b98d2df01ab3c4d")

// Create clients from source to destination ("Live" means "not mocked")
const sourceClient = Cosmos.ClientSource.Live("https://rpc.bbn-1.babylon.chain.kitchen")
const destinationClient = Evm.PublicClientDestination.Live({ chain: bob, transport: http() })
const signingClient = Cosmos.SigningClient.Live(
  "https://rpc.bbn-1.babylon.chain.kitchen",
  await DirectSecp256k1HdWallet.fromMnemonic("memo memo memo", { prefix: "bbn" }),
  { gasPrice: GasPrice.fromString("0.007ubbn") },
)
// Specify the channel over which to send funds
const sourceChannel = Cosmos.ChannelSource.Live({
  ucs03address: "bbn1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292q77945h",
  channelId: 1,
})
const destinationChannel = Evm.ChannelDestination.Live({
  ucs03address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
  channelId: 1,
})

// Build main program
const main = pipe(
  // 1. Create order instruction
  Effect.all([
    // Declare fund transfer
    FungibleAssetOrder.cosmosToEvm({
      sender: SENDER,
      receiver: RECEIVER,
      baseToken: "ubbn",
      baseAmount: 100n,
      quoteAmount: 100n,
      sourceChainId: UniversalChainId.make("babylon.bbn-1"),
      sourceChannelId: ChannelId.make(1),
    }),
    // Declare fee transfer
    FungibleAssetOrder.cosmosToEvm({
      sender: SENDER,
      receiver: RECEIVER,
      baseToken: "ubbn",
      baseAmount: 163427n,
      quoteAmount: 0n,
      sourceChainId: UniversalChainId.make("babylon.bbn-1"),
      sourceChannelId: ChannelId.make(1),
    }),
  ]),
  Effect.map(Ucs03.Batch.fromOperand),
  // 2. Send order instruction
  Effect.andThen((instruction) =>
    Cosmos.sendInstruction(
      instruction,
      "bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh",
      [
        {
          denom: "ubbn",
          amount: `${163427 + 100}`,
        },
      ],
    )
  ),
  // 3. Provide clients & channel configuration
  Effect.provide(signingClient),
  Effect.provide(sourceClient),
  Effect.provide(destinationClient),
  Effect.provide(sourceChannel),
  Effect.provide(destinationChannel),
)

// Run main program
Effect.runPromise(main)
  .then((result) => JSON.stringify(result, null, 2))
  .then(console.log)
  .catch(console.error)
