import { Effect } from "effect"
import { http, toHex } from "viem"
import { bobSepolia } from "viem/chains"
import { privateKeyToAccount } from "viem/accounts"

import {
  EvmChannelSource,
  readErc20Allowance,
  increaseErc20Allowance,
  waitForTransactionReceipt,
  ViemPublicClientSource,
  ViemPublicClient,
  createViemPublicClient,
  createViemWalletClient,
  ViemWalletClient
} from "@unionlabs/sdk/evm"
import {
  Instruction,
  sendInstructionEvm,
  createEvmToCosmosFungibleAssetOrder
} from "@unionlabs/sdk/ucs03"
import {
  CosmosChannelDestination,
  CosmWasmClientDestination,
  createCosmWasmClient
} from "@unionlabs/sdk/cosmos"
import { AddressCosmosZkgm, AddressEvmZkgm } from "@unionlabs/sdk/schema"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

// SOURCE WALLET KEY
const PRIVATE_KEY =
  process.env.PRIVATE_KEY || "0x0000000000000000000000000000000000000000000000000000000000000000"

// Define transfer parameters as constants for reuse
const SENDER = AddressEvmZkgm.make("0xfaebe5bf141cc04a3f0598062b98d2df01ab3c4d")
const RECEIVER = AddressCosmosZkgm.make(toHex("bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh"))
const MINTER_UCS03_ADDRESS = "0xe33534b7f8D38C6935a2F6Ad35E09228dA239962"

// Define token transfers
const TRANSFERS = [
  {
    sender: SENDER,
    receiver: RECEIVER,
    baseToken: "0x2be4bf88014a6574cb10df3b7826be8356aa2499", // uniBTCd on Bob
    baseAmount: 100n,
    quoteAmount: 100n
  },
  {
    sender: SENDER,
    receiver: RECEIVER,
    baseToken: "0x4200000000000000000000000000000000000006", // WETH on on BOB
    baseAmount: 50n,
    quoteAmount: 0n
  }
] as const

const createBatch = Effect.gen(function* () {
  yield* Effect.log("creating transfer 1")
  const transfer1 = yield* createEvmToCosmosFungibleAssetOrder(TRANSFERS[0])
  yield* Effect.log("creating transfer 2 (fee transfer)")
  const transferFee = yield* createEvmToCosmosFungibleAssetOrder(TRANSFERS[1])

  return Instruction.Batch.make({ operand: [transfer1, transferFee] })
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
      MINTER_UCS03_ADDRESS
    )

    yield* Effect.log(`current ${transfer.baseToken} allowance: ${currentAllowance}`)

    // If allowance is insufficient, increase it
    if (currentAllowance < transfer.baseAmount) {
      yield* Effect.log(`increasing ${transfer.baseToken} allowance...`)

      // Approve exact amount needed
      const txHash = yield* increaseErc20Allowance(
        transfer.baseToken,
        MINTER_UCS03_ADDRESS,
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
        MINTER_UCS03_ADDRESS
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
      "https://babylon-testnet-rpc.nodes.guru"
    )

    const publicSourceClient = yield* createViemPublicClient({
      chain: bobSepolia,
      transport: http()
    })

    const account = privateKeyToAccount(PRIVATE_KEY as `0x${string}`)
    const walletClient = yield* createViemWalletClient({
      account,
      chain: bobSepolia,
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

      yield* Effect.log("sending batch!")
      return yield* sendInstructionEvm(batch)
    }).pipe(
      Effect.provideService(ViemPublicClient, { client: publicSourceClient }),
      Effect.provideService(ViemPublicClientSource, { client: publicSourceClient }),
      Effect.provideService(CosmWasmClientDestination, { client: cosmWasmClientDestination }),
      Effect.provideService(CosmosChannelDestination, {
        ucs03address: "bbn15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljlspm2re6",
        channelId: 9
      }),
      Effect.provideService(EvmChannelSource, {
        ucs03address: "0xe33534b7f8D38C6935a2F6Ad35E09228dA239962",
        channelId: 1
      }),
      Effect.provideService(ViemWalletClient, {
        client: walletClient,
        account: account,
        chain: bobSepolia
      })
    )
  })
).then(e => {
  console.log(JSON.stringify(e, null, 2))
  console.log(e)
})
