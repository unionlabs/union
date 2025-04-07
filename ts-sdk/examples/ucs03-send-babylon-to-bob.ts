import { Effect } from "effect"
import { http } from "viem"
import { sepolia } from "viem/chains"
import {
  CosmWasmClientSource,
  SigningCosmWasmClientContext,
  createCosmWasmClient,
  createSigningCosmWasmClient
} from "@unionlabs/sdk/cosmos"
import { Instruction, sendInstructionCosmos, createCosmosToEvmFungibleAssetOrder } from "@unionlabs/sdk/ucs03"
import {
  EvmChannelDestination,
  ViemPublicClient,
  createViemPublicClient,
  ViemPublicClientDestination
} from "@unionlabs/sdk/evm"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { CosmosChannelSource } from "@unionlabs/sdk/cosmos"
import { Decimal } from "@cosmjs/math"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

const MNEMONIC = process.env.MNEMONIC || "memo memo memo"

const SENDER = "0x52a648ef2157fd3bafa90bbac510b9a4870fdf36"
const RECEIVER = "0xfaebe5bf141cc04a3f0598062b98d2df01ab3c4d"
const SOURCE_UCS03_ADDRESS = "bbn15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljlspm2re6"

// toHex(intent.sender),
// intent.receiver,
// toHex(intent.baseToken),

// Define token transfers
const TRANSFERS = [
  {
    sender: SENDER,
    receiver: RECEIVER,
    // baseToken: "bbn1nznl7srgmtx9juevnrasr2srslqzze3083g30w8ug96f96rjpeuqg8f6uw",
    baseToken: "ubbn",
    baseAmount: 100n,
    quoteAmount: 100n
  }
] as const

const createBatch = Effect.gen(function* () {
  yield* Effect.log("creating transfer 1")
  const transfer1 = yield* createCosmosToEvmFungibleAssetOrder(TRANSFERS[0])

  console.log(JSON.stringify(transfer1, null, 2))

  return Instruction.Batch.make({ operand: [transfer1] })
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
  //     SOURCE_UCS03_ADDRESS
  //   )

  //   yield* Effect.log(`current ${transfer.baseToken} allowance: ${currentAllowance}`)

  //   // If allowance is insufficient, increase it
  //   if (currentAllowance < transfer.baseAmount) {
  //     yield* Effect.log(`increasing ${transfer.baseToken} allowance...`)

  //     // Approve exact amount needed
  //     const txHash = yield* increaseErc20Allowance(
  //       transfer.baseToken,
  //       SOURCE_UCS03_ADDRESS,
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
  //       SOURCE_UCS03_ADDRESS
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
      yield* Effect.log("batch abi", Instruction.encodeAbi(batch))

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
        ucs03address: SOURCE_UCS03_ADDRESS,
        channelId: 1
      })
    )
  })
).then(e => {
  console.log(JSON.stringify(e, null, 2))
  console.log(e)
})
