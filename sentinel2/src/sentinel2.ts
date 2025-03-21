import { Effect, Schedule, Data, Context, Schema, Arbitrary, FastCheck } from "effect"
import { createWalletClient, http, erc20Abi } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { sepolia } from "viem/chains"
import { ViemWalletClient, writeContract , readErc20Allowance, increaseErc20Allowance, ViemPublicClientSource, createViemPublicClient, waitForTransactionReceipt, EvmChannelSource } from "@unionlabs/sdk/evm"
import { CosmWasmClientDestination, createCosmWasmClient, CosmosChannelDestination } from "@unionlabs/sdk/cosmos"
import { createEvmToCosmosFungibleAssetOrder, Batch, sendInstructionEvm, sendInstructionCosmos } from "@unionlabs/sdk/ucs03"
import yargs from "yargs"
import { hideBin } from "yargs/helpers"
import fs from "node:fs"
import type { Address } from "viem"

type Hex = `0x${string}`

// Chain pair configuration
interface ChainPair {
  sourceChain: string
  destinationChain: string
  timeframeMs: number
  enabled: boolean
}

// EVM transfer configuration
interface TransferConfig {
  enabled: boolean
  privateKey: string
  sourceChainIdEVM: string
  sourceChainIdCosmos: string // TODO: Change them later
  destinationChainId: string // TODO: Change them later
  sourceChainIdAptos: string // TODO: Change them later
  rpcs: Array<string>
  gasPriceDenom: string
  receiverAddress: Address
  denomAddress: Address
  amount_range: Array<bigint>
  cosmosAccountType: string
}

// Combined configuration shape
interface ConfigFile {
  interactions: Array<ChainPair>
  cycleIntervalMs: number
  transfers?: Array<TransferConfig>
  privkeys_for_loadtest?: Array<string>
  load_test_enabled: boolean
}

// export class Transfer extends Schema.Class<Transfer>("Transfer")({
//   token: Schema.Literal("0x09B8aE6BB8D447bF910068E6c246A270F42b41be", "0x09B8aE6BB8D447bF910068E6c246A270F42b41be"),
//   amount: Schema.Int.pipe(Schema.between(1, 80)),
//   hex: Schema.String.pipe(Schema.pattern(/^0x[0-9a-fA-F]+$/))
// }) {}

class DoTransferError extends Data.TaggedError("DoTransferError")<{
  cause: unknown
}> {}

class FilesystemError extends Data.TaggedError("FilesystemError")<{
  message: string
  cause: unknown
}> {}

export class Config extends Context.Tag("Config")<Config, { readonly config: ConfigFile }>() {}

const doTransferRetrySchedule = Schedule.exponential("2 seconds", 2.0).pipe(
  Schedule.intersect(Schedule.recurs(2)) // Limit retries to 2
)

function loadConfig(configPath: string) {
  return Effect.tryPromise({
    try: async () => {
      if (!fs.existsSync(configPath)) {
        throw new Error("Config file not found. Ensure config.json exists.")
      }
      const rawData = fs.readFileSync(configPath, "utf-8")
      const config: ConfigFile = JSON.parse(rawData)
      if (!Array.isArray(config.interactions) || config.interactions.length === 0) {
        throw new Error("Config file is invalid or interactions array is empty.")
      }
      return config
    },
    catch: error =>
      new FilesystemError({
        message: "Config file is invalid or interactions array is empty.",
        cause: error
      })
  })
}

const createBatchFromTransfers = (transfers: readonly {
  sender: Hex
  receiver: Address
  baseToken: Hex
  baseAmount: bigint
  quoteAmount: bigint
}[]) =>
  Effect.gen(function* () {
    const transferInstructions = []
    for (const transfer of transfers) {
      yield* Effect.log(`creating transfer for ${transfer.baseToken}`)
      const transferInstruction = yield* createEvmToCosmosFungibleAssetOrder(transfer)
      transferInstructions.push(transferInstruction)
    }
    return Batch(transferInstructions)
  }).pipe(Effect.withLogSpan("batch creation"))

  
const doTransfer = (task: TransferConfig) =>
  Effect.gen(function* (_) {
    let chainType: "Cosmos" | "EVM" | "Aptos"
    let sourceChainId: string

    if (task.sourceChainIdCosmos) {
      chainType = "Cosmos"
      sourceChainId = task.sourceChainIdCosmos
    } else if (task.sourceChainIdAptos) {
      chainType = "Aptos"
      sourceChainId = task.sourceChainIdAptos
    } else {
      chainType = "EVM"
      sourceChainId = task.sourceChainIdEVM
    }

    const arb = Arbitrary.make(Schema.Int.pipe(Schema.between(1, 80)))
    const random_amount = BigInt(FastCheck.sample(arb, 1)[0]!)

    yield* Effect.log(
      `\n[${chainType}] Starting transfer for chainId=${sourceChainId} to chain=${task.destinationChainId}`
    )

    const account = privateKeyToAccount(`0x${task.privateKey.replace(/^0x/, "")}`)
    const tokenAddress = task.denomAddress
    const receiver = task.receiverAddress

    const publicSourceClient = yield* createViemPublicClient({
      chain: sepolia,
      transport: http()
    })


    // const walletClient = createWalletClient({
    //   account,
    //   chain: sepolia,
    //   transport: http()
    // })


    // const tx_hash = yield* writeContract(walletClient, {
    //   account,
    //   chain: sepolia,
    //   address: tokenAddress,
    //   abi: erc20Abi,
    //   functionName: "transfer",
    //   args: [task.receiverAddress, random_amount]
    // }).pipe(
    //   Effect.provideService(ViemWalletClient, { client: walletClient }),
    //   Effect.mapError(e => e.cause.message)
    // )

    // yield* Effect.log("Transfer tx hash:", tx_hash)

    // Define hardcoded UCS03 addresses and channel ids for now
    const UCS03_ADDRESS = "0xe33534b7f8D38C6935a2F6Ad35E09228dA239962" // UCS03 contract on Sepolia
    const CHANNEL_ID = 1 // Hardcoded channel ID
    
    const TRANSFERS = [
      {
        sender: account.address,
        receiver: receiver,
        baseToken: tokenAddress, // Token to transfer
        baseAmount: random_amount,
        quoteAmount: 0n // Quote amount is not used in this case
      }
    ] as const

    yield* Effect.log("Attempting to create CosmWasm client...")
    const cosmWasmClientDestination = yield* createCosmWasmClient(
      "https://rpc.rpc-node.union-testnet-10.union.build"
    ).pipe(
      Effect.catchTag("CosmWasmClientError", (error) =>
        Effect.gen(function* () {
          yield* Effect.logError("CosmWasm client creation failed:", error)
          return yield* Effect.fail(
            new DoTransferError({ cause: error })
          )
        })
      )
    )
    
    
    yield* Effect.log("CosmWasm client created successfully")


    yield* Effect.log("Transfers to be made:", TRANSFERS)

    yield* Effect.gen(function* () {
      const batch = yield* createBatchFromTransfers(TRANSFERS)
      yield* Effect.log("Batch created:", batch)

    }).pipe(
      Effect.provideService(CosmWasmClientDestination, { client: cosmWasmClientDestination}),
      Effect.provideService(ViemPublicClientSource, { client: publicSourceClient }),
      Effect.provideService(CosmosChannelDestination, {
        ucs03address: "union15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljls7u4ry6",
        channelId: 1
      }),
      // Effect.provideService(EvmChannelSource, {
      //   ucs03address: UCS03_ADDRESS,
      //   channelId: 1
      // }),
      // Effect.provideService(ViemWalletClient, {
      //   client: walletClient,
      //   account: account,
      //   chain: sepolia
      // })
    )
    // const createBatch = Effect.gen(function* () {
    //   const transferInstructions = []
      
    //   for (const transfer of TRANSFERS) {
    //     yield* Effect.log(`creating transfer for ${transfer.baseToken}`)
    //     const transferInstruction = yield* createEvmToCosmosFungibleAssetOrder(transfer)
    //     transferInstructions.push(transferInstruction)
    //   }

    //   return Batch(transferInstructions)
    // }).pipe(Effect.withLogSpan("batch creation"))


    // yield* Effect.log("Creating batch...")
    // const batch = yield* createBatch
    // yield* Effect.log("Batch created:", batch)

    // const checkAndIncreaseAllowances = Effect.gen(function* () {
    //   yield* Effect.log("Checking token allowances...")

    //   for (const transfer of TRANSFERS) {
    //     yield* Effect.log(`checking ${transfer.baseToken} allowance...`)

    //     // Check current allowance
    //     const currentAllowance = yield* readErc20Allowance(
    //       transfer.baseToken,
    //       transfer.sender,
    //       UCS03_ADDRESS
    //     )

    //     yield* Effect.log(`current ${transfer.baseToken} allowance: ${currentAllowance}`)

    //     // If allowance is insufficient, increase it
    //     if (currentAllowance < transfer.baseAmount) {
    //       yield* Effect.log(`increasing ${transfer.baseToken} allowance...`)

    //       // Approve exact amount needed
    //       const txHash = yield* increaseErc20Allowance(
    //         transfer.baseToken,
    //         UCS03_ADDRESS,
    //         transfer.baseAmount
    //       )

    //       yield* Effect.log(`approval transaction sent: ${txHash}`)

    //       // Wait for transaction receipt
    //       // const receipt = yield* waitForTransactionReceipt(txHash)

    //       // yield* Effect.log(`approval confirmed in block: ${receipt.blockNumber}`)

    //       // Verify new allowance
    //       const newAllowance = yield* readErc20Allowance(
    //         transfer.baseToken,
    //         transfer.sender,
    //         UCS03_ADDRESS
    //       )

    //       yield* Effect.log(`new ${transfer.baseToken} allowance: ${newAllowance}`)
    //     } else {
    //       yield* Effect.log(`${transfer.baseToken} allowance is sufficient`)
    //     }
    //   }

    //   yield* Effect.log("All allowances checked and increased if needed")
    // }).pipe(Effect.withLogSpan("allowance check and increase"))

    // yield* Effect.gen(function* () {
    //   yield* Effect.log("creating batch")
    //   const batch = yield* createBatch
    //   yield* Effect.log("batch created", JSON.stringify(batch))

    //   // Check and increase allowances before sending the batch
    //   yield* Effect.log("checking and increasing allowances if needed")
    //   yield* checkAndIncreaseAllowances
    //   yield* Effect.log("allowances verified")

    //   yield* Effect.log("sending batch")
    //   return yield* sendInstructionEvm(batch)
    // }).pipe(
    //   Effect.provideService(ViemWalletClient, { client: walletClient }),
    //   Effect.provideService(ViemWalletClient, { client: walletClient }),
    //   Effect.provideService(CosmWasmClientDestination, { client: cosmWasmClientDestination }),
    //   Effect.provideService(CosmosChannelDestination, {
    //     ucs03address: UCS03_ADDRESS,
    //     channelId: CHANNEL_ID
    //   }),
    //   Effect.provideService(EvmChannelSource, {
    //     ucs03address: UCS03_ADDRESS,
    //     channelId: CHANNEL_ID
    //   }),
    //   Effect.provideService(ViemWalletClient, {
    //     client: walletClient,
    //     account: account,
    //     chain: sepolia
    //   })
    // )




  })

const transferLoop = Effect.repeat(
  Effect.gen(function* (_) {
    let config = (yield* Config).config

    const transfers: Array<TransferConfig> = config.transfers ?? []
    if (transfers.length > 0) {
      yield* Effect.log("\n========== Starting transfers tasks ==========")
      for (const task of transfers) {
        if (task.enabled) {
          yield* Effect.retry(doTransfer(task), doTransferRetrySchedule) // Retry logic for transfer
        }
      }
    } else {
      yield* Effect.log("No transfers configured. Skipping transfer step.")
    }
    yield* Effect.log("Transfers done (or skipped). Sleeping 10 minutes...")
  }),
  Schedule.spaced("3 seconds")
)

const runIbcChecksForever = Effect.repeat(
  Effect.gen(function* (_) {
    // TODO: Uncomment below. Commented out for now
    // let config = (yield* Config).config
    // const chainPairs: Array<ChainPair> = config.interactions
    // yield* Effect.log("\n========== Starting IBC cross-chain checks ==========")
    // for (const pair of chainPairs) {
    //   if (!pair.enabled) {
    //     yield* Effect.log("Checking task is disabled. Skipping.")
    //     continue
    //   }
    //   yield* Effect.log(
    //     `Checking pair ${pair.sourceChain} <-> ${pair.destinationChain} with timeframe ${pair.timeframeMs}ms`
    //   )
    //   // Simulating an IBC check
    //   if (Math.random() > 0.3) {
    //     yield* Effect.log("IBC Check successful!")
    //   } else {
    //     yield* Effect.log("IBC Check failed due to network error")
    //   }
    // }
    // yield* Effect.log("IBC Checks done (or skipped). Sleeping 10 minutes...")
  }),
  Schedule.spaced("5 seconds")
)

const mainEffect = Effect.gen(function* (_) {
  const argv = yield* Effect.sync(() =>
    yargs(hideBin(process.argv))
      .option("config", {
        alias: "c",
        type: "string",
        demandOption: true,
        describe: "Path to the configuration file"
      })
      .help()
      .alias("help", "h")
      .parseSync()
  )

  const config = yield* loadConfig(argv.config)

  yield* Effect.all([transferLoop, runIbcChecksForever], { concurrency: "unbounded" }).pipe(
    Effect.provideService(Config, { config })
  )
})

Effect.runPromise(mainEffect).catch(err => Effect.logError("Error in mainEffect", err))
