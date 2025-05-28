import { Decimal } from "@cosmjs/math"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { getFullnodeUrl } from "@mysten/sui/client"
import { AddressCosmosZkgm } from "@unionlabs/sdk/schema/address"
import { Instruction } from "@unionlabs/sdk/ucs03"
import { Effect } from "effect"
import { CosmosChannelDestination } from "../src/cosmos/channel.js"
import { CosmosChannelSource } from "../src/cosmos/channel.js"
import {
  CosmWasmClientDestination,
  CosmWasmClientSource,
  createCosmWasmClient,
  createSigningCosmWasmClient,
  SigningCosmWasmClientContext,
} from "../src/cosmos/client.js"
import { SuiChannelDestination } from "../src/sui/channel.js"
import {
  createSuiPublicClient,
  SuiPublicClient,
  SuiPublicClientDestination,
} from "../src/sui/client.js"

import { toHex } from "viem"
import {
  createCosmosToSuiFungibleAssetOrder,
  createSuiToCosmosFungibleAssetOrder,
} from "../src/ucs03/fungible-asset-order.js"
// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}
const MNEMONIC = process.env.MNEMONIC || "memo memo memo"

// Define token transfers
const TRANSFERS = [
  {
    sender: AddressCosmosZkgm.make(toHex("union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv")),
    receiver: "0x97c9e78b9c3b18f3714544e300234ea873e0904032cf3706fd4e5fd30605df7e",
    baseToken: "muno",
    baseAmount: 11n,
    quoteAmount: 11n,
  },
  {
    sender: "0x97c9e78b9c3b18f3714544e300234ea873e0904032cf3706fd4e5fd30605df7e",
    receiver: AddressCosmosZkgm.make(toHex("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2")),
    baseTokenType:
      "0x76b0a4a20519477bb4dd1dc4215cddabad5bfe92ef9f791a78507f60da07c371::fungible_token::FUNGIBLE_TOKEN",
    baseAmount: 11n,
    quoteAmount: 11n,
  },
] as const

const createFungibleAssetOrderCosmosToSui = Effect.gen(function*() {
  yield* Effect.log("creating transfer 1")
  return yield* createCosmosToSuiFungibleAssetOrder(TRANSFERS[0])
}).pipe(Effect.withLogSpan("Cosmos to sui fungible asset order creation"))

const createFungibleAssetOrderSuiToCosmos = Effect.gen(function*() {
  yield* Effect.log("creating transfer 1")
  return yield* createSuiToCosmosFungibleAssetOrder(TRANSFERS[1])
}).pipe(Effect.withLogSpan("Cosmos to sui fungible asset order creation"))

Effect.runPromiseExit(
  Effect.gen(function*() {
    const config = {
      url: getFullnodeUrl("testnet"),
    }
    const publicClient = yield* createSuiPublicClient(config)

    const sender = "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"
    const receiver = "0x97c9e78b9c3b18f3714544e300234ea873e0904032cf3706fd4e5fd30605df7e"
    const base_token = "muno"
    const baseAmount = 11n
    const quoteAmount = 11n

    const cosmWasmClientSource = yield* createCosmWasmClient(
      "https://rpc.rpc-node.union-testnet-10.union.build",
    )
    // Create a wallet from mnemonic (in a real app, use a secure method to get this)
    const wallet = yield* Effect.tryPromise(() =>
      DirectSecp256k1HdWallet.fromMnemonic(MNEMONIC, { prefix: "union" })
    )

    // Get the first account address
    const [firstAccount] = yield* Effect.tryPromise(() => wallet.getAccounts())

    // Create a signing client
    const signingClient = yield* createSigningCosmWasmClient(
      "https://rpc.rpc-node.union-testnet-10.union.build",
      wallet,
      {
        gasPrice: { amount: Decimal.fromUserInput("1", 6), denom: "muno" },
      },
    )

    yield* Effect.log("creating batch")
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
    yield* Effect.log("assetOrder created", JSON.stringify(assetOrder, null, 2))
    const encoded = Instruction.encodeAbi(assetOrder)
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
