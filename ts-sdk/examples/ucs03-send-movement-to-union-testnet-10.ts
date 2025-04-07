import { Effect } from "effect"

import { AptosConfig, Network, Ed25519PrivateKey, Account } from "@aptos-labs/ts-sdk"
import {
  AptosPublicClient,
  createAptosPublicClient,
  AptosWalletClient
} from "../src/aptos/client.js"
import { createAptosToCosmosFungibleAssetOrder } from "../src/ucs03/fungible-asset-order.js"
import { CosmWasmClientDestination, createCosmWasmClient } from "../src/cosmos/client.js"
import { Batch } from "../src/ucs03/instruction.js"
import { sendInstructionAptos } from "../src/ucs03/send-instruction.js"
import { AptosChannelSource } from "../src/aptos/channel.js"
import { CosmosChannelDestination } from "../src/cosmos/channel.js"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

const PRIVATE_KEY =
  process.env.PRIVATE_KEY || "0x0000000000000000000000000000000000000000000000000000000000000000"

// Define transfer parameters as constants for reuse
const SENDER = "0x4f597b9ac27cc279a66f4963c7955861955604eab8b38dcffee4cbcb7756e4d8"
const RECEIVER = "union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv"
const UCS03_ADDRESS = "0x80a825c8878d4e22f459f76e581cb477d82f0222e136b06f01ad146e2ae9ed84" // UCS03 contract on Movement Bardock

// Define token transfers
const TRANSFERS = [
  {
    sender: SENDER,
    receiver: RECEIVER,
    baseToken: "0x188b41399546602e35658962477fdf72bd52443474a899d9d48636e8bc299c2c", // Muno on Movement
    baseAmount: 1n,
    quoteAmount: 1n
  },
  {
    sender: SENDER,
    receiver: RECEIVER,
    baseToken: "0xfb706c831839f51ed5735e045720008d23ba4998e5a02e541af4a1bc9baabbc2", // stadv4tnt on Movement
    baseAmount: 1n,
    quoteAmount: 1n
  }
] as const

const createBatch = Effect.gen(function* () {
  yield* Effect.log("creating transfer 1")
  const transfer1 = yield* createAptosToCosmosFungibleAssetOrder(TRANSFERS[0])
  yield* Effect.log("creating transfer 2")
  const transfer2 = yield* createAptosToCosmosFungibleAssetOrder(TRANSFERS[1])
  return Batch.make({ operand: [transfer1, transfer2] })
}).pipe(Effect.withLogSpan("batch creation"))

Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create clients and setup
    yield* Effect.log("transferring from APTOS to UNION")

    yield* Effect.log("creating clients")
    const cosmWasmClientDestination = yield* createCosmWasmClient(
      "https://rpc.rpc-node.union-testnet-10.union.build"
    )
    const rpcUrl = "https://aptos.testnet.bardock.movementlabs.xyz/v1"

    const config = new AptosConfig({
      fullnode: rpcUrl,
      network: Network.CUSTOM
    })
    const publicClient = yield* createAptosPublicClient(config)

    const privateKey = new Ed25519PrivateKey(PRIVATE_KEY)
    const account = Account.fromPrivateKey({ privateKey })

    yield* Effect.log("clients created")

    // Main effect: create the batch and send it
    return yield* Effect.gen(function* () {
      yield* Effect.log("creating batch")
      const batch = yield* createBatch
      yield* Effect.log("batch created", JSON.stringify(batch))

      yield* Effect.log("sending batch")
      return yield* sendInstructionAptos(batch)
    }).pipe(
      Effect.provideService(AptosPublicClient, { client: publicClient }),
      Effect.provideService(CosmWasmClientDestination, { client: cosmWasmClientDestination }),
      Effect.provideService(CosmosChannelDestination, {
        ucs03address: "union15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljls7u4ry6",
        channelId: 1
      }),
      Effect.provideService(AptosWalletClient, { client: publicClient, account: account }),
      Effect.provideService(AptosChannelSource, {
        ucs03address: UCS03_ADDRESS,
        channelId: 1
      })
    )
  })
).then(e => {
  console.log(JSON.stringify(e, null, 2))
  console.log(e)
})
