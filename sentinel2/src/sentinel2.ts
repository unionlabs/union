import { Effect, Schedule, Data } from "effect"
import { ucs03abi } from '@unionlabs/sdk/evm/abi'
import yargs from "yargs"
import { hideBin } from "yargs/helpers"
import consola from "consola"
import fs from "node:fs"
import { type Address } from "viem"
import { argv } from "node:process"


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
  sourceChainIdCosmos: string
  destinationChainId: string
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

class DoTransferError extends Data.TaggedError("DoTransferError")<{
    cause: unknown
  }> {}

const doTransferRetrySchedule = Schedule.exponential("2 seconds", 2.0).pipe(
    Schedule.intersect(Schedule.recurs(2)), // Limit retries to 3
  )

function loadConfig(configPath: string): ConfigFile {
  if (!fs.existsSync(configPath)) {
    throw new Error("Config file not found. Ensure config.json exists.")
  }
  const rawData = fs.readFileSync(configPath, "utf-8")
  const config: ConfigFile = JSON.parse(rawData)
  if (!Array.isArray(config.interactions) || config.interactions.length === 0) {
    throw new Error("Config file is invalid or interactions array is empty.")
  }
  return config
}

const doTransfer = (transferConfig: TransferConfig) =>
  Effect.tryPromise({
    try: async () => {
        consola.info("transferConfig:", transferConfig.denomAddress)
      // Simulate transfer logic:
      // For demonstration, we randomly succeed or fail.
      if (Math.random() > 0.3) {
        consola.info("Transfer successful!")
        return "Transfer successful!"
      } else {
        consola.info("Error !!!")
        throw new Error("Transfer failed due to network error")
      }
    },
    catch: (error) => new DoTransferError({ cause: error }) // Wrap the error with DoTransferError
  })

const transferLoop = (config: ConfigFile) =>
  Effect.gen(function* (_) {
    while (true) {
        const transfers: Array<TransferConfig> = config.transfers ?? []
        const TEN_MINUTES_MS = 10 * 60 * 1000
        if (transfers.length > 0) {
            consola.info("\n========== Starting transfers tasks ==========")
            for (const task of transfers) {
                if (task.enabled){
                    yield* Effect.retry(doTransfer(task), doTransferRetrySchedule); // TODO: How to make this not crash after x failed attempts?
                }
            }
        } else {
            consola.info("No transfers configured. Skipping transfer step.")
        }

        consola.info(`Transfers done (or skipped). Sleeping 10 minutes...`)
        yield* Effect.sleep(5 * 1000) // 20 seconds delay
    }
  })

// ----- Main Effect to Load Config and Start Transfer(s) -----

const parse_argv = Effect.sync(() => {
    const argv = yargs(hideBin(process.argv))
      .option("config", {
        alias: "c",
        type: "string",
        demandOption: true,
        describe: "Path to the configuration file"
      })
      .help()
      .alias("help", "h")
      .parse()
})

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
    );


    consola.info("argv:", argv.config)
    

    const configPath = argv.config
    consola.info(`Using config file: ${configPath}`)

    const config = loadConfig(configPath)
   
    yield* transferLoop(config);

})

// ----- Run the Main Effect -----

Effect.runPromise(mainEffect).catch((err) =>
  consola.error("Error in mainEffect", err)
)
