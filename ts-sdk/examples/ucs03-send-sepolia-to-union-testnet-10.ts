import { Effect } from "effect"
import {
  ViemPublicClientSource,
  ViemPublicClient,
  createViemPublicClient,
  createViemWalletClient
} from "../src/evm/client.js"
import { http } from "viem"
import { sepolia } from "viem/chains"
import { createEvmToCosmosFungibleAssetOrder } from "../src/ucs03/fungible-asset-order.js"
import { CosmWasmClientDestination, createCosmWasmClient } from "../src/cosmos/client.js"
import { Batch } from "../src/ucs03/instruction.js"
import { sendInstructionEvm } from "../src/ucs03/send-instruction.js"
import { privateKeyToAccount } from "viem/accounts"
import { ViemWalletClient } from "../src/evm/client.js"
import { EvmChannelSource } from "../src/evm/channel.js"
import { readErc20Allowance, increaseErc20Allowance } from "../src/evm/erc20.ts"
import { waitForTransactionReceipt } from "../src/evm/receipts.ts"
import { CosmosChannelDestination } from "../src/cosmos/channel.ts"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

const PRIVATE_KEY =
  process.env.PRIVATE_KEY || "0x0000000000000000000000000000000000000000000000000000000000000000"

// Define transfer parameters as constants for reuse
const SENDER = "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA"
const RECEIVER = "stars1qcvavxpxw3t8d9j7mwaeq9wgytkf5vwputv5x4"
const UCS03_ADDRESS = "0xe33534b7f8D38C6935a2F6Ad35E09228dA239962" // UCS03 contract on Sepolia

// Define token transfers
const TRANSFERS = [
  {
    sender: SENDER,
    receiver: RECEIVER,
    baseToken: "0x779877a7b0d9e8603169ddbd7836e478b4624789", // LINK on sepolia
    baseAmount: 100n,
    quoteAmount: 100n
  },
  {
    sender: SENDER,
    receiver: RECEIVER,
    baseToken: "0x1c7d4b196cb0c7b01d743fbc6116a902379c7238", // USDC on sepolia
    baseAmount: 100n,
    quoteAmount: 100n
  },
  {
    sender: SENDER,
    receiver: RECEIVER,
    baseToken: "0x7b79995e5f793a07bc00c21412e50ecae098e7f9", // WETH on sepolia
    baseAmount: 50n,
    quoteAmount: 0n
  }
] as const

const createBatch = Effect.gen(function* () {
  yield* Effect.log("creating transfer 1")
  const transfer1 = yield* createEvmToCosmosFungibleAssetOrder(TRANSFERS[0])
  yield* Effect.log("creating transfer 2")
  const transfer2 = yield* createEvmToCosmosFungibleAssetOrder(TRANSFERS[1])
  yield* Effect.log("creating transfer 3 (fee transfer)")
  const transferFee = yield* createEvmToCosmosFungibleAssetOrder(TRANSFERS[2])

  return Batch([transfer1, transfer2, transferFee])
}).pipe(Effect.withLogSpan("batch creation"))

// Check and increase allowances if needed
const checkAndIncreaseAllowances = Effect.gen(function* () {
  yield* Effect.log("Checking token allowances...")

  for (const transfer of TRANSFERS) {
    yield* Effect.log(`checking ${transfer.baseToken} allowance...`)

    // Check current allowance
    const currentAllowance = yield* readErc20Allowance(
      transfer.baseToken,
      transfer.sender,
      UCS03_ADDRESS
    )

    yield* Effect.log(`current ${transfer.baseToken} allowance: ${currentAllowance}`)

    // If allowance is insufficient, increase it
    if (currentAllowance < transfer.baseAmount) {
      yield* Effect.log(`increasing ${transfer.baseToken} allowance...`)

      // Approve exact amount needed
      const txHash = yield* increaseErc20Allowance(
        transfer.baseToken,
        UCS03_ADDRESS,
        transfer.baseAmount
      )

      yield* Effect.log(`approval transaction sent: ${txHash}`)

      // Wait for transaction receipt
      const receipt = yield* waitForTransactionReceipt(txHash)

      yield* Effect.log(`approval confirmed in block: ${receipt.blockNumber}`)

      // Verify new allowance
      const newAllowance = yield* readErc20Allowance(
        transfer.baseToken,
        transfer.sender,
        UCS03_ADDRESS
      )

      yield* Effect.log(`new ${transfer.baseToken} allowance: ${newAllowance}`)
    } else {
      yield* Effect.log(`${transfer.baseToken} allowance is sufficient`)
    }
  }

  yield* Effect.log("All allowances checked and increased if needed")
}).pipe(Effect.withLogSpan("allowance check and increase"))

Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create clients and setup
    yield* Effect.log("transferring from sepolia to stargaze")

    yield* Effect.log("creating clients")
    const cosmWasmClientDestination = yield* createCosmWasmClient(
      "https://rpc.rpc-node.union-testnet-10.union.build"
    )

    const publicSourceClient = yield* createViemPublicClient({
      chain: sepolia,
      transport: http()
    })

    const account = privateKeyToAccount(PRIVATE_KEY)
    const walletClient = yield* createViemWalletClient({
      account,
      chain: sepolia,
      transport: http()
    })

    yield* Effect.log("clients created")

    // Main effect: create the batch and send it
    return yield* Effect.gen(function* () {
      yield* Effect.log("creating batch")
      const batch = yield* createBatch
      yield* Effect.log("batch created", JSON.stringify(batch))

      // Check and increase allowances before sending the batch
      yield* Effect.log("checking and increasing allowances if needed")
      yield* checkAndIncreaseAllowances
      yield* Effect.log("allowances verified")

      yield* Effect.log("sending batch")
      return yield* sendInstructionEvm(batch)
    }).pipe(
      Effect.provideService(ViemPublicClient, { client: publicSourceClient }),
      Effect.provideService(ViemPublicClientSource, { client: publicSourceClient }),
      Effect.provideService(CosmWasmClientDestination, { client: cosmWasmClientDestination }),
      Effect.provideService(CosmosChannelDestination, {
        ucs03address: "union15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljls7u4ry6",
        channelId: 1
      }),
      Effect.provideService(EvmChannelSource, {
        ucs03address: UCS03_ADDRESS,
        channelId: 1
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
