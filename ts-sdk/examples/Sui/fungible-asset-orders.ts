/**
 * @title Get All Coins
 * @badge WIP:caution
 */
/// <reference types="effect" />
// @paths: {"@unionlabs/sdk": ["../ts-sdk/src"], "@unionlabs/sdk/*": ["../ts-sdk/src/*"]}
// @ts-ignore
/*
BigInt["prototype"].toJSON = function() {
  return this.toString()
}
import { Decimal } from "@cosmjs/math"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { Cosmos, FungibleAssetOrder, Sui, Ucs03, Ucs05 } from "@unionlabs/sdk"
import { Effect } from "effect"
import { toHex } from "viem"
const MNEMONIC = process.env.MNEMONIC || "memo memo memo"

// Define token transfers
const TRANSFERS = [
  FungibleAssetOrder.cosmosToSui({
    sender: Ucs05.AddressCosmosZkgm.make(toHex("union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv")),
    receiver: "0x97c9e78b9c3b18f3714544e300234ea873e0904032cf3706fd4e5fd30605df7e",
    baseToken: "muno",
    baseAmount: 11n,
    quoteAmount: 11n,
  }),
  FungibleAssetOrder.suiToCosmos({
    sender: "0x97c9e78b9c3b18f3714544e300234ea873e0904032cf3706fd4e5fd30605df7e",
    receiver: Ucs05.AddressCosmosZkgm.make(toHex("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2")),
    baseTokenType:
      "0x76b0a4a20519477bb4dd1dc4215cddabad5bfe92ef9f791a78507f60da07c371::fungible_token::FUNGIBLE_TOKEN",
    baseAmount: 11n,
    quoteAmount: 11n,
  }),
] as const

Effect.runPromiseExit(
  Effect.gen(function*() {
    const publicClient = Sui.PublicClient.FromNode("testnet")

    const sender = "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"
    const receiver = "0x97c9e78b9c3b18f3714544e300234ea873e0904032cf3706fd4e5fd30605df7e"
    const base_token = "muno"
    const baseAmount = 11n
    const quoteAmount = 11n

    const cosmWasmClientSource = Cosmos.Client.Live(
      "https://rpc.rpc-node.union-testnet-10.union.build",
    )
    // Create a wallet from mnemonic (in a real app, use a secure method to get this)
    const wallet = yield* Effect.tryPromise(() =>
      DirectSecp256k1HdWallet.fromMnemonic(MNEMONIC, { prefix: "union" })
    )
    // Get the first account address
    const [firstAccount] = yield* Effect.tryPromise(() => wallet.getAccounts())

    // Create a signing client
    const signingClient = Cosmos.SigningClient.Live(
      "https://rpc.rpc-node.union-testnet-10.union.build",
      wallet,
      {
        gasPrice: { amount: Decimal.fromUserInput("1", 6), denom: "muno" },
      },
    )

    yield* Effect.log("creating batch")
    const orders = Effect.all(TRANSFERS).pipe(
      Effect.provide(publicClient),
      Effect.provide(signingClient)
      Effect.provideService(Sui.ChannelDestination, {
        ucs03address: "0xf8e63d8dd3c083d0c87554a984d14cbbf6c3b314207a7ddde035ac33ea757d8a",
        channelId: 2,
      }),
      Effect.provideService(Cosmos.ChannelSource, {
        ucs03address: "union15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljls7u4ry6",
        channelId: 1,
      }),
      Effect.provide(Cosmos.ClientDestination, { client: cosmWasmClientSource }),
      Effect.provideService(CosmosChannelDestination, {
        ucs03address: "union15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljls7u4ry6",
        channelId: 3,
      }),
    )
    const assetOrder = yield* createFungibleAssetOrderCosmosToSui.pipe(
      Effect.provideService(SigningCosmWasmClientContext, {
        client: signingClient,
        address: firstAccount.address,
      }),
      Effect.provideService(CosmWasmClientSource, { client: cosmWasmClientSource }),
      Effect.provideService(SuiPublicClientDestination, { client: publicClient }),
      Effect.provideService(SuiChannelDestination, {
        ucs03address: "0xf8e63d8dd3c083d0c87554a984d14cbbf6c3b314207a7ddde035ac33ea757d8a",
        channelId: 2,
      }),
      Effect.provideService(CosmosChannelSource, {
        ucs03address: "union15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljls7u4ry6",
        channelId: 1,
      }),
    )
    // yield* Effect.log("assetOrder created", JSON.stringify(assetOrder, null, 2))
    const encoded = Ucs03.encode(assetOrder)
    yield* Effect.log("Encoded:", encoded)

    const assetOrder2 = yield* createFungibleAssetOrderSuiToCosmos.pipe(
      Effect.provideService(SigningCosmWasmClientContext, {
        client: signingClient,
        address: firstAccount.address,
      }),
      Effect.provideService(SuiPublicClient, { client: publicClient }),
      Effect.provideService(CosmWasmClientDestination, { client: cosmWasmClientSource }),
      Effect.provideService(CosmosChannelDestination, {
        ucs03address: "union15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljls7u4ry6",
        channelId: 3,
      }),
      Effect.catchAllCause(cause => {
        console.error("cause is:", cause)
      }),
    )
    yield* Effect.log("assetOrder2 created", JSON.stringify(assetOrder2, null, 2))
    const encoded2 = Instruction.encodeAbi(assetOrder2)
    yield* Effect.log("Encoded:", encoded2)
  }),
).then(exit => console.log(JSON.stringify(exit, null, 2)))

// ---cut---
*/
