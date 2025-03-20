import { Effect, Schedule, Data } from "effect"
import { createWalletClient, createPublicClient, http, erc20Abi } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { sepolia } from "viem/chains"
import { ViemWalletClient, writeContract } from "@unionlabs/sdk/evm"
import yargs from "yargs"
import { hideBin } from "yargs/helpers"
import fs from "node:fs"
import type { Address } from "viem"

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

function getRandomArbitrary(min_bigint: bigint, max_bigint: bigint) {
  const min = Number(min_bigint)
  const max = Number(max_bigint)
  const value = Math.random() * (max - min) + min

  return BigInt(Math.ceil(value))
}

class DoTransferError extends Data.TaggedError("DoTransferError")<{
  cause: unknown
}> {}

class FilesystemError extends Data.TaggedError("FilesystemError")<{
  message: string
  cause: unknown
}> {}

const doTransferRetrySchedule = Schedule.exponential("2 seconds", 2.0).pipe(
  Schedule.intersect(Schedule.recurs(2)) // Limit retries to 3
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

    const random_amount = getRandomArbitrary(task.amount_range[0] ?? 1n, task.amount_range[1] ?? 1n)

    yield* Effect.log(
      `\n[${chainType}] Starting transfer for chainId=${sourceChainId} to chain=${task.destinationChainId}`
    )

    const client = createPublicClient({
      chain: sepolia,
      transport: http()
    })

    const account = privateKeyToAccount(`0x${task.privateKey.replace(/^0x/, "")}`)
    const tokenAddress = task.denomAddress

    const walletClient = createWalletClient({
      account,
      chain: sepolia,
      transport: http()
    })

    const tx_hash = yield* writeContract(walletClient, {
      account,
      chain: sepolia,
      address: tokenAddress,
      abi: erc20Abi,
      functionName: "transfer",
      args: [task.receiverAddress, random_amount]
    }).pipe(
      Effect.provideService(ViemWalletClient, { client: walletClient }),
      Effect.mapError(e => e.cause.message)
    )

    yield* Effect.log("Transfer tx hash:", tx_hash)
  })

const transferLoop = (config: ConfigFile) =>
  Effect.repeat(
    Effect.gen(function* (_) {
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
    Schedule.spaced(10 * 30 * 10) // Sleep for 10 minutes between each loop
  )

const runIbcChecksForever = (config: ConfigFile) =>
  Effect.repeat(
    Effect.gen(function* (_) {
      const chainPairs: Array<ChainPair> = config.interactions
      yield* Effect.log("\n========== Starting IBC cross-chain checks ==========")
      for (const pair of chainPairs) {
        if (!pair.enabled) {
          yield* Effect.log("Checking task is disabled. Skipping.")
          continue
        }
        yield* Effect.log(
          `Checking pair ${pair.sourceChain} <-> ${pair.destinationChain} with timeframe ${pair.timeframeMs}ms`
        )

        // Simulating an IBC check
        if (Math.random() > 0.3) {
          yield* Effect.log("IBC Check successful!")
        } else {
          yield* Effect.log("IBC Check failed due to network error")
        }
      }
      yield* Effect.log("IBC Checks done (or skipped). Sleeping 10 minutes...")
    }),
    Schedule.spaced(5 * 1000) // Sleep for 5 seconds between each loop
  )

// const mainEffect = (configPath: string) => Effect.gen(function* (_) {
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

  yield* Effect.log("argv:", argv.config)

  const configPath = argv.config
  yield* Effect.log(`Using config file: ${configPath}`)

  const configEffect = loadConfig(configPath)

  const config = yield* configEffect // This will resolve to the loaded config or an error

  const transferEffect = transferLoop(config)
  const ibcCheckEffect = runIbcChecksForever(config)

  yield* Effect.forkAll([transferEffect, ibcCheckEffect])
  yield* Effect.zip(transferEffect, ibcCheckEffect)
})

// ----- Run the Main Effect -----

Effect.runPromise(mainEffect).catch(err => Effect.logError("Error in mainEffect", err))
