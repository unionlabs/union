// CosmWasm
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { GasPrice } from "@cosmjs/stargate"
// EVM
import { http, toHex } from "viem"
import { bobSepolia } from "viem/chains"
// Union
import { Cosmos, Evm, FungibleAssetOrder, Ucs05 } from "@unionlabs/sdk"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { Effect, pipe } from "effect"

// We will send funds from sender to receiver
const SENDER = Ucs05.AddressCosmosZkgm.make(toHex("bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh"))
const RECEIVER = Ucs05.AddressEvmZkgm.make("0xfaebe5bf141cc04a3f0598062b98d2df01ab3c4d")

const wallet = await DirectSecp256k1HdWallet.fromMnemonic(process.env.MNEMONIC as string, {
  prefix: "bbn",
})

const accounts = await wallet.getAccounts()

console.log({ accounts })

// Create clients from source to destination ("Live" means "not mocked")
const sourceClient = Cosmos.ClientSource.Live("https://rpc.bbn-1.babylon.chain.kitchen")
const destinationClient = Evm.PublicClientDestination.Live({ chain: bobSepolia, transport: http() })
const signingClient = Cosmos.SigningClient.Live(
  "https://rpc.bbn-1.babylon.chain.kitchen",
  // await DirectSecp256k1HdWallet.fromMnemonic("memo memo memo", { prefix: "bbn" }),
  await DirectSecp256k1HdWallet.fromMnemonic(process.env.MNEMONIC as string, { prefix: "bbn" }),
  { gasPrice: GasPrice.fromString("0.000006ubbn") },
)
// Specify the channel over which to send funds
const sourceChannel = Cosmos.ChannelSource.Live({
  // @ts-ignore
  ucs03address: "bbn15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljlspm2re6",
  channelId: 1,
})
const destinationChannel = Evm.ChannelDestination.Live({
  ucs03address: "0xe33534b7f8D38C6935a2F6Ad35E09228dA239962",
  channelId: 1,
})

// Build main program
const main = pipe(
  // 1. Create order instruction
  FungibleAssetOrder.cosmosToEvm({
    sender: SENDER,
    receiver: RECEIVER,
    baseToken: "ubbn",
    baseAmount: 100n,
    quoteAmount: 100n,
    sourceChainId: UniversalChainId.make("babylon.bbn-1"),
    sourceChannelId: ChannelId.make(1),
  }),
  // 2. Send order instruction
  Effect.andThen((instruction) => Cosmos.sendInstruction(instruction, SENDER)),
  // 3. Provide clients & channel configuration
  Effect.provide(signingClient),
  Effect.provide(sourceClient),
  Effect.provide(destinationClient),
  Effect.provide(sourceChannel),
  Effect.provide(destinationChannel),
)

// Run main program
Effect.runPromise(main)
  .then(console.log)
  .catch(console.error)
