import { Effect } from "effect"
import {
  ViemPublicClient,
  createViemPublicClient,
  ViemPublicClientDestination
} from "../src/evm/client.js"
import { http } from "viem"
import { sepolia } from "viem/chains"
import { createCosmosToEvmFungibleAssetOrder } from "../src/ucs03/fungible-asset-order.js"
import {
  CosmWasmClientSource,
  SigningCosmWasmClientContext,
  createCosmWasmClient,
  createSigningCosmWasmClient
} from "../src/cosmos/client.js"
import { Batch, encodeAbi } from "../src/ucs03/instruction.js"
import { sendInstructionCosmos } from "../src/ucs03/send-instruction.js"
import { EvmChannelDestination } from "../src/evm/channel.js"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { CosmosChannelSource } from "../src/cosmos/channel.ts"
import { Decimal } from "@cosmjs/math"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

const MNEMONIC = process.env.MNEMONIC || "memo memo memo"

const SENDER = "union1d95n4r6dnrfrps59szhl8mk7yqewsuzyw0zh5q"
const RECEIVER = "0xE6831e169d77a861A0E71326AFA6d80bCC8Bc6aA"
const UCS03_ADDRESS = "union15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljls7u4ry6" // UCS03 contract on testnet 10

// Define token transfers
const TRANSFERS = [
  {
    sender: SENDER,
    receiver: RECEIVER,
    baseToken: "muno",
    baseAmount: 100n,
    quoteAmount: 100n
  }
] as const

const createBatch = Effect.gen(function* () {
  yield* Effect.log("creating transfer 1")
  const transfer1 = yield* createCosmosToEvmFungibleAssetOrder(TRANSFERS[0])

  return Batch([transfer1])
}).pipe(Effect.withLogSpan("batch creation"))

// Check and increase allowances if needed
const checkAndIncreaseAllowances = Effect.gen(function* () {
  const publicClient = (yield* ViemPublicClient).client

  yield* Effect.log("Checking token allowances...")

  // for (const transfer of TRANSFERS) {
  //   yield* Effect.log(`checking ${transfer.baseToken} allowance...`)

  //   // Check current allowance
  //   const currentAllowance = yield* readErc20Allowance(
  //     transfer.baseToken,
  //     transfer.sender,
  //     UCS03_ADDRESS
  //   )

  //   yield* Effect.log(`current ${transfer.baseToken} allowance: ${currentAllowance}`)

  //   // If allowance is insufficient, increase it
  //   if (currentAllowance < transfer.baseAmount) {
  //     yield* Effect.log(`increasing ${transfer.baseToken} allowance...`)

  //     // Approve exact amount needed
  //     const txHash = yield* increaseErc20Allowance(
  //       transfer.baseToken,
  //       UCS03_ADDRESS,
  //       transfer.baseAmount
  //     )

  //     yield* Effect.log(`approval transaction sent: ${txHash}`)

  //     // Wait for transaction receipt
  //     const receipt = yield* waitForTransactionReceipt(txHash)

  //     yield* Effect.log(`approval confirmed in block: ${receipt.blockNumber}`)

  //     // Verify new allowance
  //     const newAllowance = yield* readErc20Allowance(
  //       transfer.baseToken,
  //       transfer.sender,
  //       UCS03_ADDRESS
  //     )

  //     yield* Effect.log(`new ${transfer.baseToken} allowance: ${newAllowance}`)
  //   } else {
  //     yield* Effect.log(`${transfer.baseToken} allowance is sufficient`)
  //   }
  // }

  yield* Effect.log("All allowances checked and increased if needed")
}).pipe(Effect.withLogSpan("allowance check and increase"))

Effect.runPromiseExit(
  Effect.gen(function* () {
    // Create clients and setup
    yield* Effect.log("transferring from sepolia to stargaze")

    yield* Effect.log("creating clients")
    const cosmWasmClientSource = yield* createCosmWasmClient(
      "https://rpc.rpc-node.union-testnet-10.union.build"
    )

    const publicDestinationClient = yield* createViemPublicClient({
      chain: sepolia,
      transport: http()
    })

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

      // Check and increase allowances before sending the batch
      // yield* Effect.log("checking and increasing allowances if needed")
      // yield* checkAndIncreaseAllowances
      // yield* Effect.log("allowances verified")

      yield* Effect.log("sending batch")
      return yield* sendInstructionCosmos(batch)
    }).pipe(
      Effect.provideService(SigningCosmWasmClientContext, {
        client: signingClient,
        address: firstAccount.address
      }),
      Effect.provideService(CosmWasmClientSource, { client: cosmWasmClientSource }),
      Effect.provideService(ViemPublicClientDestination, { client: publicDestinationClient }),
      Effect.provideService(EvmChannelDestination, {
        ucs03address: "0xe33534b7f8D38C6935a2F6Ad35E09228dA239962",
        channelId: 1
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
