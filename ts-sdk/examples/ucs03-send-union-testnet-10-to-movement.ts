import { Effect } from "effect"
import { AptosConfig, Network } from "@aptos-labs/ts-sdk"
import { AptosPublicClientDestination, createAptosPublicClient } from "../src/aptos/client.js"
import { createCosmosToAptosFungibleAssetOrder } from "../src/ucs03/fungible-asset-order.js"
import {
  CosmWasmClientSource,
  SigningCosmWasmClientContext,
  createCosmWasmClient,
  createSigningCosmWasmClient
} from "../src/cosmos/client.js"
import { Batch, encodeAbi } from "../src/ucs03/instruction.js"
import { AptosChannelDestination } from "../src/aptos/channel.js"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { CosmosChannelSource } from "../src/cosmos/channel.ts"
import { Decimal } from "@cosmjs/math"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

const MNEMONIC = process.env.MNEMONIC || "memo memo memo"

const SENDER = "union1d95n4r6dnrfrps59szhl8mk7yqewsuzyw0zh5q"
const RECEIVER = "0x4f597b9ac27cc279a66f4963c7955861955604eab8b38dcffee4cbcb7756e4d8"
const UCS03_ADDRESS = "union15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljls7u4ry6" // UCS03 contract on testnet 10

// Define token transfers
const TRANSFERS = [
  {
    sender: SENDER,
    receiver: RECEIVER,
    baseToken: "muno",
    baseAmount: 1n,
    quoteAmount: 1n
  }
] as const

const createBatch = Effect.gen(function* () {
  yield* Effect.log("creating transfer 1")
  const transfer1 = yield* createCosmosToAptosFungibleAssetOrder(TRANSFERS[0])

  return Batch([transfer1])
}).pipe(Effect.withLogSpan("batch creation"))

Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create clients and setup
    yield* Effect.log("transferring from sepolia to stargaze")

    yield* Effect.log("creating clients")
    const cosmWasmClientSource = yield* createCosmWasmClient(
      "https://rpc.rpc-node.union-testnet-10.union.build"
    )

    const rpcUrl = "https://aptos.testnet.bardock.movementlabs.xyz/v1"

    const config = new AptosConfig({
      fullnode: rpcUrl,
      network: Network.CUSTOM
    })
    const publicDestinationClient = yield* createAptosPublicClient(config)

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
        gasPrice: { amount: Decimal.fromUserInput("1", 6), denom: "muno" }
      }
    )

    yield* Effect.log("clients created")

    // Main effect: create the batch and send it
    return yield* Effect.gen(function* () {
      yield* Effect.log("creating batch")
      const batch = yield* createBatch
      yield* Effect.log("batch created", JSON.stringify(batch, null, 2))
      yield* Effect.log("batch abi", encodeAbi(batch))

      // yield* Effect.log("sending batch")
      // return yield* sendInstructionCosmos(batch)
    }).pipe(
      Effect.provideService(SigningCosmWasmClientContext, {
        client: signingClient,
        address: firstAccount.address
      }),
      Effect.provideService(CosmWasmClientSource, { client: cosmWasmClientSource }),
      Effect.provideService(AptosPublicClientDestination, { client: publicDestinationClient }),
      Effect.provideService(AptosChannelDestination, {
        ucs03address: "0x80a825c8878d4e22f459f76e581cb477d82f0222e136b06f01ad146e2ae9ed84",
        channelId: 2
      }),
      Effect.provideService(CosmosChannelSource, {
        ucs03address: UCS03_ADDRESS,
        channelId: 1
      })
    )
  })
).then(e => {
  console.log(JSON.stringify(e, null, 2))
  console.log(e)
})
