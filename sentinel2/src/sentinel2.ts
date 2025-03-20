import { Effect, Schedule, Data, Context, Schema, Arbitrary, FastCheck } from "effect"
import { createWalletClient, http, erc20Abi } from "viem"
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
    let config = (yield* Config).config
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
