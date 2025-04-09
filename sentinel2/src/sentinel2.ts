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
import { request, gql } from "graphql-request"

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

interface Packet {
  packet_send_timestamp: string | null
  packet_recv_timestamp: string | null
  write_ack_timestamp: string | null
  packet_ack_timestamp: string | null
  packet_send_transaction_hash?: string | null
  packet_recv_transaction_hash?: string | null
  write_ack_transaction_hash?: string | null
  packet_ack_transaction_hash?: string | null
  sort_order: string
}

interface HasuraResponse {
  v1_ibc_union_packets: Packet[]
}

// Combined configuration shape
interface ConfigFile {
  interactions: Array<ChainPair>
  cycleIntervalMs: number
  transfers?: Array<TransferConfig>
}

// A module-level set to keep track of already reported packet send transaction hashes.
const reportedSendTxHashes = new Set<string>()
// Variable to track sleep cycles
let sleepCycleCount = 0

const HASURA_ENDPOINT = "https://development.graphql.union.build/v1/graphql"

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


/**
 * fetchPacketsUntilCutoff
 *
 * This helper function pages through packets—starting from the most recent (the first query)
 * and then using the sort_order cursor (via the Next query) until it encounters a packet whose
 * packet_send_timestamp is earlier than the provided cutoff timestamp.
 *
 * @param srcChain The source chain identifier.
 * @param dstChain The destination chain identifier.
 * @param cutoffTimestamp A string ISO date (e.g. "2025-04-09T06:44:46.971Z") acting as the lower bound.
 *                        Only packets with a send timestamp >= cutoffTimestamp will be saved.
 * @returns An Effect that resolves to an array of Packet.
 */
const fetchPacketsUntilCutoff = (srcChain: string, dstChain: string, cutoffTimestamp: string) =>
  Effect.gen(function* () {
    let allPackets: Packet[] = []
    let cursor: string | undefined = undefined
    let continueFetching = true

    while (continueFetching) {
      let response: any

      if (!cursor) {
        // First query: no cursor (assumes API returns the most recent packets).
        const queryFirst = gql`
          query First($srcChain: String!, $dstChain: String!) {
            v2_packets(args: {
              p_source_universal_chain_id: $srcChain,
              p_destination_universal_chain_id: $dstChain
            }) {
              packet_send_timestamp
              packet_recv_timestamp
              write_ack_timestamp
              packet_ack_timestamp
              packet_send_transaction_hash
              packet_recv_transaction_hash
              write_ack_transaction_hash
              packet_ack_transaction_hash
              sort_order
            }
          }
        `
        response = yield* Effect.tryPromise({
          try: () => request(HASURA_ENDPOINT, queryFirst, { srcChain, dstChain }),
          catch: (error) => { 
            console.info("Error in first query:", error)
            throw error 
          }
        })
      } else {
        // Next query: use the last sort_order as a cursor.
        const queryNext = gql`
          query Next($sortOrder: String!, $srcChain: String!, $dstChain: String!) {
            v2_packets(args: {
              p_source_universal_chain_id: $srcChain,
              p_destination_universal_chain_id: $dstChain,
              p_sort_order: $sortOrder
            }) {
              packet_send_timestamp
              packet_recv_timestamp
              write_ack_timestamp
              packet_ack_timestamp
              packet_send_transaction_hash
              packet_recv_transaction_hash
              write_ack_transaction_hash
              packet_ack_transaction_hash
              sort_order
            }
          }
        `
        response = yield* Effect.tryPromise({
          try: () => request(HASURA_ENDPOINT, queryNext, { sortOrder: cursor, srcChain, dstChain }),
          catch: (error) => { 
            console.info("Error in second query:", error)
            throw error 
          }
        })
      }

      const currentPage: Packet[] = response?.v2_packets || []
      if (currentPage.length === 0) break

      for (const packet of currentPage) {
        // If the packet's send timestamp is missing, include it (or decide otherwise).
        if (packet.packet_send_timestamp) {
          const packetTime = new Date(packet.packet_send_timestamp).getTime()
          const cutoffTime = new Date(cutoffTimestamp).getTime()
          // Stop paging once we encounter a packet older than the cutoff.
          if (packetTime < cutoffTime) {
            continueFetching = false
            break
          }
        }
        allPackets.push(packet)
      }

      // Set cursor for the next page based on the last packet.
      if (continueFetching) {
        cursor = currentPage[currentPage.length - 1]!.sort_order
      }
    }

    return allPackets
  })


/**
 * checkPackets
 *
 * This effectful function fetches IBC packet data from Hasura and then verifies that:
 *
 *  - Packets older than the provided timeframe have a valid reception, write_ack, and ack.
 *  - If a packet’s timestamp differences exceed the provided SLA timeframe, it logs an error.
 *  - It avoids duplicate logging by tracking already reported packet send transaction hashes.
 *
 * @param sourceChain - The source chain identifier
 * @param destinationChain - The destination chain identifier
 * @param timeframeMs - The maximum allowed timeframe (in milliseconds) for the packet to be confirmed
 */
export const checkPackets = (sourceChain: string, destinationChain: string, timeframeMs: number) =>
  Effect.gen(function* () {
    const now = Date.now()
    const searchRangeMs = timeframeMs * 10
    const sinceDate = new Date(now - searchRangeMs).toISOString()

    yield* Effect.log(
      `Querying Hasura for packets >= ${sinceDate}, chain-pair: ${sourceChain} <-> ${destinationChain}`
    )   

    const now_as_date = new Date(now).toISOString()
    yield* Effect.log(
      `now: ${now_as_date}`
    )
    
    const first_packets: Packet[] = yield* fetchPacketsUntilCutoff(sourceChain, destinationChain, sinceDate)
    const second_packets: Packet[] = yield* fetchPacketsUntilCutoff(destinationChain, sourceChain, sinceDate)
    const packets = [...first_packets, ...second_packets]
    yield* Effect.log(`Fetched ${packets.length} packets from Hasura`)
        // Process each packet.
        for (const p of packets) {
          if (!p.packet_send_timestamp) continue
            const sendTimeMs = new Date(p.packet_send_timestamp).getTime()
            // Only process packets that are older than the allowed timeframe.
            if (now - sendTimeMs < timeframeMs) continue

            const sendTxHash = p.packet_send_transaction_hash ?? "?"
            if (reportedSendTxHashes.has(sendTxHash)) continue
            const sort_order_tx = p.sort_order.split("-")[1]

            // 1) RECV check.
            if (p.packet_recv_timestamp) {
              const recvTimeMs = new Date(p.packet_recv_timestamp).getTime()
              if (recvTimeMs - sendTimeMs > timeframeMs) {
                // yield* Effect.log(
                //   `[RECV TOO LATE] >${timeframeMs}ms. send_time=${p.packet_send_timestamp}, recv_time=${p.packet_recv_timestamp}, sendTxHash=${sendTxHash}`
                // )
                // reportedSendTxHashes.add(sendTxHash)
              }
            } else {
              yield* Effect.log(
                `[TRANSFER_ERROR: RECV MISSING] >${timeframeMs}ms since send. sendTxHash=${sendTxHash}, chain_pair${sourceChain}<->${destinationChain}, url: https://staging.app2.union.build/explorer/transfers/${sort_order_tx}`
              )
              reportedSendTxHashes.add(sendTxHash)
              continue
            }

            // 2) WRITE_ACK check.
            if (p.write_ack_timestamp) {
              // const writeAckTimeMs = new Date(p.write_ack_timestamp).getTime()
              // if (writeAckTimeMs - sendTimeMs > timeframeMs) {
              //   yield* Effect.log(
              //     `[TRANSFER_ERROR: WRITE_ACK TOO LATE] >${timeframeMs}ms. send_time=${p.packet_send_timestamp}, write_ack_time=${p.write_ack_timestamp}, sendTxHash=${sendTxHash}`
              //   )
              //   reportedSendTxHashes.add(sendTxHash)
              // }
            } else {
              yield* Effect.log(
                `[TRANSFER_ERROR: WRITE_ACK MISSING] >${timeframeMs}ms since send. sendTxHash=${sendTxHash}, chain_pair${sourceChain}<->${destinationChain}, url: https://staging.app2.union.build/explorer/transfers/${sort_order_tx}`
              )
              reportedSendTxHashes.add(sendTxHash)
              continue
            }

            // 3) ACK check.
            if (p.packet_ack_timestamp) {
              // const ackTimeMs = new Date(p.packet_ack_timestamp).getTime()
              // if (ackTimeMs - sendTimeMs > timeframeMs) {
              //   yield* Effect.log(
              //     `[TRANSFER_ERROR: ACK TOO LATE] >${timeframeMs}ms. send_time=${p.packet_send_timestamp}, ack_time=${p.packet_ack_timestamp}, sendTxHash=${sendTxHash}`
              //   )
              //   reportedSendTxHashes.add(sendTxHash)
              // }
            } else {
              yield* Effect.log(
                `[TRANSFER_ERROR: ACK MISSING] >${timeframeMs}ms since send. sendTxHash=${sendTxHash}, chain_pair${sourceChain}<->${destinationChain}, url: https://staging.app2.union.build/explorer/transfers/${sort_order_tx}`
              )
              reportedSendTxHashes.add(sendTxHash)
            }
          }
        
  }).pipe(Effect.withLogSpan("checkPackets"))


  const runIbcChecksForever = Effect.gen(function* (_) {
    const { config } = yield* Config
  
    const schedule = Schedule.spaced(`${config.cycleIntervalMs / 1000 / 60} minutes`)
  
    const effectToRepeat = Effect.gen(function* (_) {
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
  
        yield* checkPackets(
          pair.sourceChain,
          pair.destinationChain,
          pair.timeframeMs
        )
      }
  
      yield* Effect.log(`IBC Checks done (or skipped). Sleeping ${config.cycleIntervalMs / 1000 / 60} minutes...`)
      sleepCycleCount++
      if (sleepCycleCount % 10 === 0) {
        reportedSendTxHashes.clear()
        yield* Effect.log("Cleared reported block hashes.")
      }
    })
  
    return yield* Effect.repeat(effectToRepeat, schedule)
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
  )

  const config = yield* loadConfig(argv.config)

  yield* Effect.all([/*transferLoop, */runIbcChecksForever], { concurrency: "unbounded" }).pipe(
    Effect.provideService(Config, { config })
  )
})

Effect.runPromise(mainEffect).catch(err => Effect.logError("Error in mainEffect", err))
