import { Effect, Schedule, Data } from "effect"
import { ucs03abi } from '@unionlabs/sdk/evm/abi'
import { createWalletClient, createPublicClient, http, erc20Abi } from "viem"
import { privateKeyToAccount } from "viem/accounts"
import { sepolia } from "viem/chains"
import { ViemPublicClient, ViemWalletClient, writeContract  } from '@unionlabs/sdk/evm'
import yargs from "yargs"
import { hideBin } from "yargs/helpers"
import consola from "consola"
import fs from "node:fs"
import { type Address } from "viem"
import { argv } from "node:process"
import { run } from "node:test"


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

const doTransfer = (task: TransferConfig) =>
  Effect.tryPromise({
    try: async () => {
        let chainType: "Cosmos" | "EVM" | "Aptos"

        let sourceChainId: string // TODO: change type later
        
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

        consola.info(
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
        

          writeContract(walletClient, {
            account,
            chain: sepolia,
            address: tokenAddress,
            abi: erc20Abi,
            functionName: "transfer",
            args: [task.receiverAddress, random_amount]
          })
            .pipe(
              Effect.provideService(ViemWalletClient, { client: walletClient }),
              Effect.mapError(e => e.cause.message),
              Effect.runPromiseExit
            )
            .then(exit => console.log(JSON.stringify(exit, null, 2)))
    
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


const runIbcChecksForever = (config: ConfigFile) =>
    Effect.gen(function* (_) {
        const chainPairs: Array<ChainPair> = config.interactions
        while (true) {
            consola.info("\n========== Starting IBC cross-chain checks ==========")
            for (const pair of chainPairs) {
              if (!pair.enabled) {
                consola.info("Checking task is disabled. Skipping.")
                continue
              }
              consola.info(
                `Checking pair ${pair.sourceChain} <-> ${pair.destinationChain} with timeframe ${pair.timeframeMs}ms`
              )
        
                // Simulating an IBC check
                if (Math.random() > 0.3) {
                consola.info("IBC Check successful!")
                } else {
                consola.info("IBC Check failed due to network error")
                }
            }
            consola.info(`IBC Checks done (or skipped). Sleeping 10 minutes...`)
            yield* Effect.sleep(5 * 1000) // 5 seconds delay before repeating the check
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
   
    yield* transferLoop(config)
  // Fork both effects: transferLoop and runIbcChecksForever to run concurrently
//   const transferEffect = Effect.fork(transferLoop(config))
//   const ibcCheckEffect = Effect.fork(runIbcChecksForever(config))

//   const forked = yield* Effect.forkAll([transferEffect, ibcCheckEffect]) // Fork both effects to run concurrently
    // Effect.runFork(forked) // Run the forked effects
//   yield* Effect.zip(transferEffect, ibcCheckEffect) // Ensures both effects continue indefinitely

})

// ----- Run the Main Effect -----

Effect.runPromise(mainEffect).catch((err) =>
  consola.error("Error in mainEffect", err)
)
