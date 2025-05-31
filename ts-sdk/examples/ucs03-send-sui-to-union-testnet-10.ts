import { Decimal } from "@cosmjs/math"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { getFullnodeUrl } from "@mysten/sui/client"
import { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"
import { Transaction } from "@mysten/sui/transactions"
import { AddressCosmosZkgm } from "@unionlabs/sdk/schema/address"
import { Instruction } from "@unionlabs/sdk/ucs03"
import { Effect } from "effect"
import { toHex } from "viem"
import { CosmosChannelDestination } from "../src/cosmos/channel.js"
import { SuiChannelSource } from "../src/sui/channel.js"
import { createSuiPublicClient, SuiPublicClient, SuiWalletClient } from "../src/sui/client.js"
import { writeContract } from "../src/sui/contract.js"
import { SuiFungibleAssetOrderDetails } from "../src/sui/fungible_asset_order_details.js"
import {
  createCosmosToSuiFungibleAssetOrder,
  createSuiToCosmosFungibleAssetOrder,
} from "../src/ucs03/fungible-asset-order.js"

import { effect } from "effect/Layer"
import {
  CosmWasmClientDestination,
  CosmWasmClientSource,
  createCosmWasmClient,
  createSigningCosmWasmClient,
  SigningCosmWasmClientContext,
} from "../src/cosmos/client.js"
import { sendInstructionSui } from "../src/ucs03/send-instruction.js"

// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}
const MNEMONIC = process.env.MNEMONIC || "memo memo memo"

const SOURCE_UCS03_ADDRESS = "0xbd3eb037fda14d910c5574b8c950222cc5c4211675b9c2265c07a66cef3a7691"

const SUI_RPC = "https://fullnode.testnet.sui.io:443"
const UNION_RPC = "http://localhost:26657"

// Define token transfers
const TRANSFERS = [
  {
    sender: "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779",
    receiver: AddressCosmosZkgm.make(toHex("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2")),
    baseTokenType:
      "0xd32f121aec92e5179398e21ab9beb366d854b6f985bb326266228271c3697c95::fungible_token::FUNGIBLE_TOKEN",
    baseAmount: 100n,
    quoteAmount: 100n,
  },
] as const

const createAssetOrder = Effect.gen(function*() {
  yield* Effect.log("creating transfer 1")
  const transfer1 = yield* createSuiToCosmosFungibleAssetOrder(TRANSFERS[0])
  yield* Effect.log("Created transfer", JSON.stringify(transfer1, null, 2))
  return transfer1
}).pipe(Effect.withLogSpan("Transfer Creation"))

Effect.runPromiseExit(
  Effect.gen(function*() {
    const config = {
      url: getFullnodeUrl("testnet"),
    }

    const publicClient = yield* createSuiPublicClient(config)
    const cosmWasmClientSource = yield* createCosmWasmClient(
      UNION_RPC,
    )
    // Create a wallet from mnemonic (in a real app, use a secure method to get this)
    const wallet = yield* Effect.tryPromise(() =>
      DirectSecp256k1HdWallet.fromMnemonic(MNEMONIC, { prefix: "union" })
    )

    // Get the first account address
    const [firstAccount] = yield* Effect.tryPromise(() => wallet.getAccounts())

    // Create a signing client
    const signingClient = yield* createSigningCosmWasmClient(
      UNION_RPC,
      wallet,
      {
        gasPrice: { amount: Decimal.fromUserInput("1", 6), denom: "muno" },
      },
    )

    const keypair = Ed25519Keypair.deriveKeypair(MNEMONIC)

    const transfer = yield* createAssetOrder.pipe(
      Effect.provideService(CosmWasmClientSource, { client: cosmWasmClientSource }),
      Effect.provideService(SigningCosmWasmClientContext, { client: signingClient }),
      Effect.provideService(SuiPublicClient, { client: publicClient }),
      Effect.provideService(CosmWasmClientDestination, { client: cosmWasmClientSource }),
      Effect.provideService(CosmosChannelDestination, {
        ucs03address: "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c",
        channelId: 2,
      }),
      Effect.catchAllCause(cause =>
        Effect.sync(() => {
          console.error("cause is:", cause)
        })
      ),
    )

    const returned = yield* sendInstructionSui(transfer).pipe(
      Effect.provideService(SuiWalletClient, { client: publicClient, signer: keypair }),
      Effect.provideService(SuiChannelSource, {
        ucs03address: SOURCE_UCS03_ADDRESS,
        channelId: 2,
      }),
      Effect.provideService(SuiFungibleAssetOrderDetails, {
        typename_t:
          "0xd32f121aec92e5179398e21ab9beb366d854b6f985bb326266228271c3697c95::fungible_token::FUNGIBLE_TOKEN",
        ibc_store: "0x97a40c1954f94607c473a03e67890a566e7b8d75e562a2f93ab468ece12b34e3",
        relay_store: "0x79f9dcae544c9ba9272e9b0eebb0e8f0abe4fd5c5971d2f87929584b050a48b2",
        coin: "0x929ede1bef34a18d5cd7b69196eb2bcac980d1e940b66dc81f6f2d5ee7ebc547",
        metadata: "0x2d0cd827c09c8e9a36ce4f8b23ff45ff82d204bb975982b7dbbefe24f6c475a5",
      }),
      Effect.catchAllCause(cause =>
        Effect.sync(() => {
          console.error("cause is:", cause)
        })
      ),
    )
    return returned
  }),
).then(exit => console.log(JSON.stringify(exit, null, 2)))
