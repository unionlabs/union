#!/usr/bin/env node
declare global {
  interface BigInt {
    toJSON: () => string
  }
}

if (!BigInt.prototype.toJSON) {
  Object.defineProperty(BigInt.prototype, "toJSON", {
    value: function () {
      return this.toString()
    },
    writable: true,
    configurable: true
  })
}

import { request, gql } from "graphql-request"
import fetch, { Headers } from "node-fetch"
import fs from "fs"
import yargs from "yargs"
import { hideBin } from "yargs/helpers"
import consola from "consola"

// For the EVM cross-chain transfer snippet:
import { Address, fallback, http } from "viem"
import { bech32, hex, bytes } from "@scure/base"
import { holesky, sepolia } from "viem/chains"
import { DirectSecp256k1Wallet } from "@cosmjs/proto-signing"
import { privateKeyToAccount } from "viem/accounts"
// If you’re pulling createUnionClient from your local or a published package:
import {
  ChainId,
  CosmosChainId,
  createUnionClient,
  EvmChainId,
  type TransferAssetsParameters,
  hexToBytes
} from "@unionlabs/client"

// Hasura endpoint
const HASURA_ENDPOINT = "https://hubble-purple.hasura.app/v1/graphql"

// Set to track reported block hashes
const reportedBlockHashes = new Set<string>()

// Variable to track sleep cycles
let sleepCycleCount = 0

// Shape of the chain pair config
interface ChainPair {
  sourceChain: string
  destinationChain: string
  timeframeMs: number
}

// Shape of the EVM transfer config
interface TransferConfig {
  privateKey: string
  sourceChainIdEVM: EvmChainId
  sourceChainIdCosmos: CosmosChainId
  destinationChainId: ChainId
  rpcs: string[]
  gasPriceDenom: string
  receiverAddress: Address
  denomAddress: Address
  amount: bigint
  cosmosAccountType: string
}

// Combined config shape
interface ConfigFile {
  interactions: ChainPair[]
  cycleIntervalMs: number
  transfers?: TransferConfig[] // optional array
}

// The shape of Hasura’s response
interface HasuraResponse {
  v1_ibc_union_packets: Packet[]
}

// Define our expected shape from Hasura
interface Packet {
  packet_send_timestamp: string | null
  packet_recv_timestamp: string | null
  write_ack_timestamp: string | null
  packet_ack_timestamp: string | null

  source_chain_id: string
  destination_chain_id: string

  // optional fields
  packet_send_block_hash?: string | null
  packet_recv_block_hash?: string | null
}

// Set global fetch and Headers
if (!globalThis.fetch) {
  globalThis.fetch = fetch as any
}
if (!globalThis.Headers) {
  globalThis.Headers = Headers as any
}

// Parse command-line arguments
function loadConfig(configPath: string): ConfigFile {
  if (!fs.existsSync(configPath)) {
    throw new Error("Config file not found. Ensure config.json exists.")
  }

  const rawData = fs.readFileSync(configPath, "utf-8")
  const config = JSON.parse(rawData)

  if (!Array.isArray(config.interactions) || config.interactions.length === 0) {
    throw new Error("Config file is invalid or interactions array is empty.")
  }

  return config
}

/**
 * Check IBC packets between source_chain <-> destination_chain.
 *
 * We fetch the last 200× 'timeframe' from Hasura.
 * For each packet older than timeframe:
 *   - Check RECV / WRITE_ACK / ACK existence & timings.
 *   - Log any that are missing or exceed the time window.
 *
 * @param sourceChain A string ID (e.g. "11155111")
 * @param destinationChain Another string ID (e.g. "17000")
 * @param timeframeMs The SLA timeframe in milliseconds
 */
export async function checkPackets(
  sourceChain: string,
  destinationChain: string,
  timeframeMs: number
): Promise<void> {
  // Current time
  const now = Date.now()

  // We'll query more than the timeframe to ensure we catch all
  const searchRangeMs = timeframeMs * 2
  const sinceDate = new Date(now - searchRangeMs).toISOString()

  consola.info(
    `Querying Hasura for packets >= ${sinceDate}, chain-pair: ${sourceChain} <-> ${destinationChain}`
  )

  // Build the GraphQL query:
  const query = gql`
    query ($since: timestamptz!, $srcChain: String!, $dstChain: String!) {
      v1_ibc_union_packets(
        where: {
          _and: [
            {
              _or: [
                {
                  source_chain_id: { _eq: $srcChain }
                  destination_chain_id: { _eq: $dstChain }
                }
                {
                  source_chain_id: { _eq: $dstChain }
                  destination_chain_id: { _eq: $srcChain }
                }
              ]
            }
            { packet_send_timestamp: { _gte: $since } }
          ]
        }
        order_by: { packet_send_timestamp: asc }
      ) {
        packet_send_timestamp
        packet_recv_timestamp
        write_ack_timestamp
        packet_ack_timestamp
        source_chain_id
        destination_chain_id
        packet_send_block_hash
        packet_recv_block_hash
      }
    }
  `

  const variables = {
    since: sinceDate,
    srcChain: sourceChain,
    dstChain: destinationChain
  }

  try {
    // Post to Hasura
    const response = await request<HasuraResponse>(HASURA_ENDPOINT, query, variables)
    const data = response.v1_ibc_union_packets ?? []

    consola.info(
      `Found ${data.length} packets in the last ${searchRangeMs}ms for ${sourceChain} <-> ${destinationChain}`
    )

    // Check each packet
    for (const p of data) {
      const sendStr = p.packet_send_timestamp
      if (!sendStr) {
        continue
      }

      // Convert sendStr to a Date
      const sendTimeMs = new Date(sendStr).getTime()
      // Only check those older than or equal to SLA timeframe
      if (now - sendTimeMs < timeframeMs) {
        // Not old enough to be considered overdue
        continue
      }

      // If we're here, the packet is older than `timeframeMs`.
      const recvStr = p.packet_recv_timestamp
      const writeAckStr = p.write_ack_timestamp
      const ackStr = p.packet_ack_timestamp

      const blockHash = p.packet_send_block_hash ?? "?"

      if (reportedBlockHashes.has(blockHash)) {
        continue
      }

      // 1) RECV
      if (!recvStr) {
        consola.error(
          `[RECV MISSING] >${timeframeMs}ms since send. BlockHash=${blockHash}, source_chain=${p.source_chain_id}, dest_chain=${p.destination_chain_id}`
        )
        reportedBlockHashes.add(blockHash)
        continue
      } else {
        const recvTimeMs = new Date(recvStr).getTime()
        if (recvTimeMs - sendTimeMs > timeframeMs) {
          consola.error(
            `[RECV TOO LATE] >${timeframeMs}ms. send_time=${sendStr}, recv_time=${recvStr}, blockHash=${blockHash}`
          )
          reportedBlockHashes.add(blockHash)
        }
      }

      // 2) WRITE_ACK
      if (!writeAckStr) {
        consola.error(
          `[WRITE_ACK MISSING] >${timeframeMs}ms since send. BlockHash=${blockHash}, source_chain=${p.source_chain_id}, dest_chain=${p.destination_chain_id}`
        )
        reportedBlockHashes.add(blockHash)
        continue
      } else {
        const writeAckTimeMs = new Date(writeAckStr).getTime()
        if (writeAckTimeMs - sendTimeMs > timeframeMs) {
          consola.error(
            `[WRITE_ACK TOO LATE] >${timeframeMs}ms. blockHash=${blockHash}, send_time=${sendStr}, write_ack_time=${writeAckStr}`
          )
          reportedBlockHashes.add(blockHash)
        }
      }

      // 3) ACK
      if (!ackStr) {
        consola.error(
          `[ACK MISSING] >${timeframeMs}ms since send. BlockHash=${blockHash}, source_chain=${p.source_chain_id}, dest_chain=${p.destination_chain_id}`
        )
        reportedBlockHashes.add(blockHash)
      } else {
        const ackTimeMs = new Date(ackStr).getTime()
        if (ackTimeMs - sendTimeMs > timeframeMs) {
          consola.error(
            `[ACK TOO LATE] >${timeframeMs}ms. send_time=${sendStr}, ack_time=${ackStr}, blockHash=${blockHash}`
          )
          reportedBlockHashes.add(blockHash)
        }
      }
    }
  } catch (error: any) {
    consola.error("Error fetching data from Hasura:", error.message)
  }
}

/**
 * Perform an EVM cross-chain transfer or estimate the gas for it.
 * Adapt the logic as needed to match your chain IDs / workflow.
 */
async function doTransfer(task: TransferConfig) {
  const isCosmosChain = Boolean(task.sourceChainIdCosmos)
  const chainType = isCosmosChain ? "Cosmos" : "EVM"
  try {
    consola.info(
      "\n[%s] Starting transfer for chainId=%s to chain=%s",
      chainType,
      isCosmosChain ? task.sourceChainIdCosmos : task.sourceChainIdEVM,
      task.destinationChainId
    )

    const evmAccount = privateKeyToAccount(`0x${task.privateKey.replace(/^0x/, "")}`)
    const cosmosAccount = await DirectSecp256k1Wallet.fromKey(
      Uint8Array.from(hexToBytes(task.privateKey)),
      task.cosmosAccountType
    )

    const transports = task.rpcs.map(rpc => http(rpc))
    const sourceChainId = isCosmosChain ? task.sourceChainIdCosmos : task.sourceChainIdEVM

    const unionClient = isCosmosChain
      ? createUnionClient({
          account: cosmosAccount,
          chainId: task.sourceChainIdCosmos,
          gasPrice: { amount: "0.025", denom: task.gasPriceDenom },
          transport: transports[0]
        })
      : createUnionClient({
          account: evmAccount,
          chainId: task.sourceChainIdEVM,
          transport: fallback(transports)
        })

    const transactionPayload = isCosmosChain
      ? ({
          amount: BigInt(task.amount),
          denomAddress: task.denomAddress,
          destinationChainId: task.destinationChainId,
          receiver: task.receiverAddress
        } satisfies TransferAssetsParameters<typeof sourceChainId>)
      : ({
          amount: task.amount,
          denomAddress: task.denomAddress,
          destinationChainId: task.destinationChainId,
          receiver: task.receiverAddress,
          autoApprove: true
        } satisfies TransferAssetsParameters<typeof sourceChainId>)

    const transferResp = await unionClient.transferAsset(transactionPayload)
    if (transferResp.isErr()) {
      consola.error("[%s] Transfer error:", chainType, transferResp.error)
      return
    }

    consola.info("[%s] Transfer success:", chainType, transferResp.value)
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error)
    consola.error("[%s] Transfer exception: %s", chainType, msg)
  }
}

/**
 * This loop runs your IBC checks on the interval specified by `config.cycleIntervalMs`.
 * (For example, once every hour if config.cycleIntervalMs = 3600000)
 */
async function runIbcChecksForever(config: ConfigFile) {
  const chainPairs: ChainPair[] = config.interactions

  while (true) {
    consola.info("\n========== Starting IBC cross-chain checks ==========")
    for (const pair of chainPairs) {
      consola.info(
        `Checking pair ${pair.sourceChain} <-> ${pair.destinationChain} with timeframe ${pair.timeframeMs}ms`
      )
      try {
        await checkPackets(pair.sourceChain, pair.destinationChain, pair.timeframeMs)
        consola.info(`Check complete for pair ${pair.sourceChain} <-> ${pair.destinationChain}`)
      } catch (err) {
        consola.error(
          `Error while checking pair ${pair.sourceChain} <-> ${pair.destinationChain}:`,
          err
        )
      }
    }

    // Optionally clear the reportedBlockHashes set every 3 cycles
    sleepCycleCount++
    if (sleepCycleCount % 3 === 0) {
      reportedBlockHashes.clear()
      consola.info("Cleared reported block hashes.")
    }

    // Sleep for whatever cycleIntervalMs is set to (e.g. 1 hour)
    consola.info(`IBC checks done. Sleeping for ${config.cycleIntervalMs / 1000 / 60} minutes...`)
    await new Promise(resolve => setTimeout(resolve, config.cycleIntervalMs))
  }
}

/**
 * This loop runs your transfer tasks every 10 minutes,
 * regardless of how often IBC checks happen.
 */
async function runTransfersForever(config: ConfigFile) {
  const transfers: TransferConfig[] = config.transfers ?? []
  const TEN_MINUTES_MS = 10 * 60 * 1000

  while (true) {
    if (transfers.length > 0) {
      consola.info("\n========== Starting transfers tasks ==========")
      for (const task of transfers) {
        await doTransfer(task)
      }
    } else {
      consola.info("No transfers configured. Skipping transfer step.")
    }

    consola.info(`Transfers done (or skipped). Sleeping 10 minutes...`)
    await new Promise(resolve => setTimeout(resolve, TEN_MINUTES_MS))
  }
}

/**
 * Kick off both loops in parallel.
 */
async function main() {
  const argv = await yargs(hideBin(process.argv))
    .option("config", {
      alias: "c",
      type: "string",
      demandOption: true,
      describe: "Path to the configuration file"
    })
    .help()
    .alias("help", "h")
    .parse()

  const configPath = argv.config
  consola.info(`Using config file: ${configPath}`)

  // Load configuration
  const config = loadConfig(configPath)

  // Start both infinite loops in parallel:
  await Promise.all([runIbcChecksForever(config), runTransfersForever(config)])
}

// Just call `main()` immediately
main().catch(err => consola.error("Error in main()", err))
