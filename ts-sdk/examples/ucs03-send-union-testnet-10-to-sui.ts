import { getFullnodeUrl } from "@mysten/sui/client"
import { Effect } from "effect"
import { createSuiPublicClient, SuiPublicClient, SuiPublicClientDestination, SuiWalletClient } from "../src/sui/client.js"
import { Ed25519Keypair } from '@mysten/sui/keypairs/ed25519';
import { Transaction } from '@mysten/sui/transactions';
import { writeContract } from "../src/sui/contract.js"
import { SuiChannelDestination, SuiChannelSource } from "../src/sui/channel.js"
import { AddressCosmosZkgm } from "@unionlabs/sdk/schema/address"
import { SuiFungibleAssetOrderDetails } from "../src/sui/fungible_asset_order_details.js"
import { Instruction, sendInstructionCosmos } from "@unionlabs/sdk/ucs03"
import { toHex } from "viem"
import { Decimal } from "@cosmjs/math"
import {
  createCosmosToSuiFungibleAssetOrder,
  createSuiToCosmosFungibleAssetOrder,
} from "../src/ucs03/fungible-asset-order.js"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { CosmosChannelDestination, CosmosChannelSource } from "../src/cosmos/channel.js"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { bech32, bytes, hex } from "@scure/base"

import { sendInstructionSui } from "../src/ucs03/send-instruction.js"
import {
  CosmWasmClientDestination,
  CosmWasmClientSource,
  createCosmWasmClient,
  createSigningCosmWasmClient,
  SigningCosmWasmClientContext,
} from "../src/cosmos/client.js"
import { effect } from "effect/Layer";
export function hexToBytes(hexString: string): Uint8Array {
  return bytes("hex", hexString.indexOf("0x") === 0 ? hexString.slice(2) : hexString)
}
// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}
const PRIVATE_KEY = process.env.PRIVATE_KEY
  || "0x0000000000000000000000000000000000000000000000000000000000000000"

const SOURCE_UCS03_ADDRESS = "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c"

const SUI_RPC = "https://fullnode.testnet.sui.io:443"
const UNION_RPC = "http://localhost:26657"

// Define token transfers
const TRANSFERS = [
  {
    sender: AddressCosmosZkgm.make(toHex("union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2")),
    receiver: "0x835e6a7d0e415c0f1791ae61241f59e1dd9d669d59369cd056f02b3275f68779",
    baseToken: "union1tksllvfmfw9ajys27zlzx69wj5j0022ayhrwmql97nkefmhrqm4q4akuhq",
    baseAmount: 11n,
    quoteAmount: 11n
  }
] as const

const createBatch = Effect.gen(function*() {
  yield* Effect.log("creating transfer 1")
  const transfer1 = yield* createCosmosToSuiFungibleAssetOrder(TRANSFERS[0])
  yield* Effect.log("Created transfer", JSON.stringify(transfer1, null, 2))
  return transfer1
}).pipe(Effect.withLogSpan("Batch Creation"))


Effect.runPromiseExit(
  Effect.gen(function*() {
    const config = {
      url: getFullnodeUrl("testnet"),
    }

    const publicClient = yield* createSuiPublicClient(config)
    const cosmWasmClientSource = yield* createCosmWasmClient(
      UNION_RPC,
    )
    
    console.info("Divining wallet. Priv:", PRIVATE_KEY)
    const wallet = yield* Effect.tryPromise(() =>
      DirectSecp256k1Wallet.fromKey(Uint8Array.from(hexToBytes(PRIVATE_KEY)), "Union")
    ).pipe(
      Effect.catchAllCause(cause =>
        Effect.sync(() => {
          console.error("Failed to create wallet:", cause);
      })),
    )
    console.info("Wallet created")
    // Get the first account address
    const [firstAccount] = yield* Effect.tryPromise(() => wallet.getAccounts())
    console.info("Got wallet account", firstAccount.address)

    // Create a signing client
    const signingClient = yield* createSigningCosmWasmClient(
      UNION_RPC,
      wallet,
      {
        gasPrice: { amount: Decimal.fromUserInput("1", 6), denom: "muno" },
      },
    )
    
    const batch = yield* createBatch.pipe(
      Effect.provideService(CosmWasmClientSource, { client: cosmWasmClientSource }),
      Effect.provideService(SigningCosmWasmClientContext, { client: signingClient, address: firstAccount.address }),
      Effect.provideService(SuiPublicClientDestination, { client: publicClient }),
      Effect.provideService(CosmWasmClientDestination, { client: cosmWasmClientSource }),
      Effect.provideService(SuiChannelDestination, {
        ucs03address: "0xbd3eb037fda14d910c5574b8c950222cc5c4211675b9c2265c07a66cef3a7691",
        channelId: 2,
      }),
      Effect.catchAllCause(cause =>
        Effect.sync(() => {
          console.error("cause is:", cause);
      }))
    )
    yield* sendInstructionCosmos(batch, [
      {
        denom: "muno",
        amount: TRANSFERS[0].baseAmount.toString(),
      }
    ]).pipe(
      Effect.provideService(SigningCosmWasmClientContext, { client: signingClient, address: firstAccount.address }),
      Effect.provideService(CosmosChannelSource, {
            ucs03address: SOURCE_UCS03_ADDRESS,
            channelId: 2,
          }),
      )
    // yield* sendInstructionSui(transfer).pipe(
    //   Effect.provideService(SuiWalletClient, { client: publicClient, signer: keypair }),
    //   Effect.provideService(SuiChannelSource, {
    //     ucs03address: SOURCE_UCS03_ADDRESS,
    //     channelId: 2,
    //   }),
    //   Effect.provideService(SuiFungibleAssetOrderDetails, {
    //     typename_t: "0xd32f121aec92e5179398e21ab9beb366d854b6f985bb326266228271c3697c95::fungible_token::FUNGIBLE_TOKEN",
    //     ibc_store: "0x97a40c1954f94607c473a03e67890a566e7b8d75e562a2f93ab468ece12b34e3",
    //     relay_store: "0x79f9dcae544c9ba9272e9b0eebb0e8f0abe4fd5c5971d2f87929584b050a48b2",
    //     coin: "0xca7fb93d690a8ce20764421c378eb42daada699af8bba22eca2b54539619c390",
    //     metadata: "0x2d0cd827c09c8e9a36ce4f8b23ff45ff82d204bb975982b7dbbefe24f6c475a5"
    //   }),
    //   Effect.catchAllCause(cause =>
    //     Effect.sync(() => {
    //       console.error("cause is:", cause);
    //   }))
    // )
  }),
).then(exit => console.log(JSON.stringify(exit, null, 2)))
