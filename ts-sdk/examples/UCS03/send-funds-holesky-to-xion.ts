/**
 * @title Send Funds Holesky → Xion
 * @description Example transfer from Holesky to Xion.
 * @badge ✓:success
 */
/// <reference types="effect" />
/// <reference types="viem" />
// @paths: {"@unionlabs/sdk": ["../ts-sdk/src"], "@unionlabs/sdk/*": ["../ts-sdk/src/*"]}
// @ts-ignore
if (typeof BigInt.prototype.toJSON !== "function") {
  // @ts-ignore
  BigInt.prototype.toJSON = function() {
    return this.toString()
  }
}
// ---cut---
// EVM
import { http, toHex } from "viem"
import { holesky } from "viem/chains"
// Union
import { Cosmos, Evm, FungibleAssetOrder, Ucs05 } from "@unionlabs/sdk"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { TokenRawDenom } from "@unionlabs/sdk/schema/token"
import { Effect, pipe } from "effect"
import { privateKeyToAccount } from "viem/accounts"

// We will send funds from sender to receiver
const SENDER = Ucs05.AddressEvmZkgm.make("0xfaebe5bf141cc04a3f0598062b98d2df01ab3c4d")
const RECEIVER = Ucs05.AddressCosmosZkgm.make(toHex("xion122ny3mep2l7nhtafpwav2y9e5jrslhekkyv629"))

// Create clients from source to destination ("Live" means "not mocked")
const sourceClient = Evm.PublicClientSource.Live({
  chain: holesky,
  transport: http("https://rpc.17000.ethereum.chain.kitchen"),
})
const destinationClient = Cosmos.Client.Live(
  "https://rpc.xion-testnet-2.xion.chain.kitchen/",
)
const walletClient = Evm.WalletClient.Live({
  account: privateKeyToAccount("0x..."),
  chain: holesky,
  transport: http("https://rpc.17000.ethereum.chain.kitchen"),
})
// Specify the channel over which to send funds
const sourceChannel = Evm.ChannelSource.Live({
  ucs03address: "0x5fbe74a283f7954f10aa04c2edf55578811aeb03",
  channelId: 4,
})
const destinationChannel = Cosmos.ChannelDestination.Live({
  ucs03address: "xion1336jj8ertl8h7rdvnz4dh5rqahd09cy0x43guhsxx6xyrztx292qlzhdk9",
  channelId: 1,
})

// Build main program
const main = pipe(
  // 1. Create order instruction
  FungibleAssetOrder.evmToCosmos({
    sender: SENDER,
    receiver: RECEIVER,
    baseToken: TokenRawDenom.make("0x685ce6742351ae9b618f383883d6d1e0c5a31b4b"),
    baseAmount: 100n,
    quoteAmount: 100n,
    sourceChainId: UniversalChainId.make("ethereum.17000"),
    sourceChannelId: ChannelId.make(4),
  }),
  // 2. Send order instruction
  Effect.andThen(Evm.sendInstruction),
  // 3. Provide clients & channel configuration
  Effect.provide(sourceClient),
  Effect.provide(walletClient),
  Effect.provide(destinationClient),
  Effect.provide(sourceChannel),
  Effect.provide(destinationChannel),
)

// Run main program
Effect.runPromise(main)
  .then(console.log)
  .catch(console.error)
