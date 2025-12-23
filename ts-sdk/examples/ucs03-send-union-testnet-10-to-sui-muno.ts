import { Decimal } from "@cosmjs/math"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { getFullnodeUrl } from "@mysten/sui/client"
import type { Ed25519Keypair } from "@mysten/sui/keypairs/ed25519"
import { Transaction } from "@mysten/sui/transactions"
import { bech32, bytes, hex } from "@scure/base"
import { AddressCosmosZkgm } from "@unionlabs/sdk/schema/address"
import { Instruction, sendInstructionCosmos } from "@unionlabs/sdk/ucs03"
import { Effect } from "effect"
import { toHex } from "viem"
import { CosmosChannelDestination, CosmosChannelSource } from "../src/cosmos/channel.js"
import { SuiChannelDestination, SuiChannelSource } from "../src/sui/channel.js"
import {
  createSuiPublicClient,
  SuiPublicClient,
  SuiPublicClientDestination,
  SuiWalletClient,
} from "../src/sui/client.js"
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
    baseToken: "muno",
    baseAmount: 321n,
    quoteAmount: 321n,
  },
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
          console.error("Failed to create wallet:", cause)
        })
      ),
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
      Effect.provideService(SigningCosmWasmClientContext, {
        client: signingClient,
        address: firstAccount.address,
      }),
      Effect.provideService(SuiPublicClientDestination, { client: publicClient }),
      Effect.provideService(CosmWasmClientDestination, { client: cosmWasmClientSource }),
      Effect.provideService(SuiChannelDestination, {
        ucs03address: "0xbd3eb037fda14d910c5574b8c950222cc5c4211675b9c2265c07a66cef3a7691",
        channelId: 2,
      }),
      Effect.catchAllCause(cause =>
        Effect.sync(() => {
          console.error("cause is:", cause)
        })
      ),
    )
    yield* sendInstructionCosmos(batch, [
      {
        denom: "muno",
        amount: TRANSFERS[0].baseAmount.toString(),
      },
    ]).pipe(
      Effect.provideService(SigningCosmWasmClientContext, {
        client: signingClient,
        address: firstAccount.address,
      }),
      Effect.provideService(CosmosChannelSource, {
        ucs03address: SOURCE_UCS03_ADDRESS,
        channelId: 2,
      }),
    )
  }),
).then(exit => console.log(JSON.stringify(exit, null, 2)))
