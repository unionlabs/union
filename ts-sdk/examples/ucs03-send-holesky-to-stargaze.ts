import { Effect } from "effect"
import { ViemPublicClientSource } from "../src/evm/client.js"
import { createPublicClient, createWalletClient, http } from "viem"
import { sepolia } from "viem/chains"
import { CosmosDestinationConfig } from "../src/cosmos/quote-token.js"
import { createEvmToCosmosFungibleAssetOrder } from "../src/ucs03/fungible-asset-order.js"
import { CosmWasmClientDestination, createCosmWasmClient } from "../src/cosmos/client.js"
import { Batch } from "../src/ucs03/instruction.js"
import { sendInstructionEvm } from "../src/ucs03/send-instruction.js"
import { privateKeyToAccount } from "viem/accounts"
import { ViemWalletClient } from "../src/evm/client.js"
import { SourceConfig } from "../src/evm/quote-token.js"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

const PRIVATE_KEY =
  process.env.PRIVATE_KEY || "0x0000000000000000000000000000000000000000000000000000000000000000"

const createBatch = Effect.gen(function* () {
  const transfer1 = yield* createEvmToCosmosFungibleAssetOrder({
    sender: "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA",
    receiver: "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4",
    baseToken: "0x779877a7b0d9e8603169ddbd7836e478b4624789", // LINK on sepolia
    baseAmount: 100n,
    quoteAmount: 100n
  })

  const transfer2 = yield* createEvmToCosmosFungibleAssetOrder({
    sender: "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA",
    receiver: "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4",
    baseToken: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238", // USDC on sepolia
    baseAmount: 100n,
    quoteAmount: 100n
  })

  const transferFee = yield* createEvmToCosmosFungibleAssetOrder({
    sender: "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA",
    receiver: "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4",
    baseToken: "0x7b79995e5f793a07bc00c21412e50ecae098e7f9", // WETH on sepolia
    baseAmount: 50n,
    quoteAmount: 0n
  })

  return Batch([transfer1, transfer2, transferFee])
})

Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create clients and setup
    yield* Effect.log("transfering from sepolia to stargaze")

    yield* Effect.log("creating clients")
    const cosmWasmClientDestination = yield* createCosmWasmClient(
      "https://rpc.elgafar-1.stargaze-apis.com"
    )
    const account = privateKeyToAccount(PRIVATE_KEY)
    const walletClient = createWalletClient({
      account,
      chain: sepolia,
      transport: http("https://rpc.11155111.sepolia.chain.kitchen")
    })
    yield* Effect.log("clients created")

    // Main effect: create the batch and send it
    return yield* Effect.gen(function* () {
      yield* Effect.log("creating batch")
      const batch = yield* createBatch
      yield* Effect.log("batch created", JSON.stringify(batch))
      yield* Effect.log("sending batch")
      return yield* sendInstructionEvm(batch, "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA")
    }).pipe(
      Effect.provideService(ViemPublicClientSource, {
        client: createPublicClient({
          chain: sepolia,
          transport: http()
        })
      }),
      Effect.provideService(CosmWasmClientDestination, { client: cosmWasmClientDestination }),
      Effect.provideService(CosmosDestinationConfig, {
        ucs03address: "stars1x2jzeup7uwfxjxxrtfna2ktcugltntgu6kvc0eeayk0d82l247cqsnqksg",
        channelId: 3
      }),
      Effect.provideService(SourceConfig, {
        ucs03address: "0x84f074c15513f15baea0fbed3ec42f0bd1fb3efa",
        channelId: 11
      }),
      Effect.provideService(ViemWalletClient, {
        client: walletClient,
        account: account,
        chain: sepolia
      })
    )
  })
).then(e => {
  console.log(JSON.stringify(e, null, 2))
  console.log(e)
})
