#!/usr/bin/env node
import { request, gql } from "graphql-request"
import fetch, { Headers } from "node-fetch"
import fs from "fs"
import yargs from "yargs"
import { hideBin } from "yargs/helpers"

// For the EVM cross-chain transfer snippet:
import { Address, fallback, http } from "viem"
import { holesky, sepolia } from "viem/chains"
import { privateKeyToAccount } from "viem/accounts"
// If you’re pulling createUnionClient from your local or a published package:
import { createUnionClient, type TransferAssetsParameters } from "@unionlabs/client"
// Alternatively, adapt the import path to your code’s actual location if it’s local

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
interface EvmTransferConfig {
  privateKey: string
  estimateGas: boolean
  destinationChainId: string
  rpc: string
  linkContractAddress: Address
}

// Combined config shape
interface ConfigFile {
  interactions: ChainPair[]
  cycleIntervalMs: number
  evmTransfers?: EvmTransferConfig[] // optional array
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
  const searchRangeMs = timeframeMs * 500
  const sinceDate = new Date(now - searchRangeMs).toISOString()

  console.info(
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
                },
                {
                  source_chain_id: { _eq: $dstChain }
                  destination_chain_id: { _eq: $srcChain }
                }
              ]
            },
            { packet_send_timestamp: { _gte: $since } }
          ]
        }
        order_by: { packet_send_timestamp: asc }
        limit: 500
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

    console.info(
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
        console.error(
          `[RECV MISSING] >${timeframeMs}ms since send. BlockHash=${
            p.packet_send_block_hash ?? "?"
          }, source_chain=${p.source_chain_id}, dest_chain=${p.destination_chain_id}`
        )
        reportedBlockHashes.add(blockHash)
        continue
      } else {
        const recvTimeMs = new Date(recvStr).getTime()
        if (recvTimeMs - sendTimeMs > timeframeMs) {
          console.error(
            `[RECV TOO LATE] >${timeframeMs}ms. send_time=${sendStr}, recv_time=${recvStr}, blockHash=${
              p.packet_send_block_hash ?? "?"
            }`
          )
          reportedBlockHashes.add(blockHash)
        }
      }

      // 2) WRITE_ACK
      if (!writeAckStr) {
        console.error(
          `[WRITE_ACK MISSING] >${timeframeMs}ms since send. BlockHash=${
            p.packet_send_block_hash ?? "?"
          }, source_chain=${p.source_chain_id}, dest_chain=${p.destination_chain_id}`
        )
        reportedBlockHashes.add(blockHash)
        continue
      } else {
        const writeAckTimeMs = new Date(writeAckStr).getTime()
        if (writeAckTimeMs - sendTimeMs > timeframeMs) {
          console.error(
            `[WRITE_ACK TOO LATE] >${timeframeMs}ms. blockHash=${
              p.packet_send_block_hash ?? "?"
            }, send_time=${sendStr}, write_ack_time=${writeAckStr}`
          )
          reportedBlockHashes.add(blockHash)
        }
      }

      // 3) ACK
      if (!ackStr) {
        console.error(
          `[ACK MISSING] >${timeframeMs}ms since send. BlockHash=${
            p.packet_send_block_hash ?? "?"
          }, source_chain=${p.source_chain_id}, dest_chain=${p.destination_chain_id}`
        )
        reportedBlockHashes.add(blockHash)
      } else {
        const ackTimeMs = new Date(ackStr).getTime()
        if (ackTimeMs - sendTimeMs > timeframeMs) {
          console.error(
            `[ACK TOO LATE] >${timeframeMs}ms. send_time=${sendStr}, ack_time=${ackStr}, blockHash=${
              p.packet_send_block_hash ?? "?"
            }`
          )
          reportedBlockHashes.add(blockHash)
        } else {
          console.debug(`Packet fully acked on time. blockHash=${p.packet_send_block_hash ?? "?"}`)
        }
      }
    }
  } catch (error: any) {
    console.error("Error fetching data from Hasura:", error.message)
  }
}

const LINK_CONTRACT_ADDRESS = "0x685cE6742351ae9b618F383883D6d1e0c5A31B4B"
const RECEIVER = "0x153919669Edc8A5D0c8D1E4507c9CE60435A1177"

/**
 * Perform an EVM cross-chain transfer or estimate the gas for it.
 * Adapt the logic as needed to match your chain IDs / workflow.
 */
async function doEvmTransfer(task: EvmTransferConfig) {
  try {
    console.info(`\n[EVMTx] Starting EVM transfer for chainId=${task.destinationChainId}`)

    // The account derived from the private key
    const evmAccount = privateKeyToAccount(`0x${task.privateKey.replace(/^0x/, "")}`)
    console.info("[EVMTx] EVM account:", evmAccount)
    const unionClient = createUnionClient({
      chainId: "17000",
      account: evmAccount,
      transport: fallback([
        http("https://rpc.holesky.sepolia.chain.kitchen"),
        http(holesky?.rpcUrls.default.http.at(0))
      ])
    })

    // Construct transaction payload
    const transactionPayload = {
      amount: 421n,
      destinationChainId: `${sepolia.id}`,
      receiver: RECEIVER,
      denomAddress: LINK_CONTRACT_ADDRESS,
      autoApprove: true
    } satisfies TransferAssetsParameters<"17000">

    console.log("transactionPayload: ", transactionPayload)

    // Simulate to get gas estimation
    const gasEstimationResponse = await unionClient.simulateTransaction(transactionPayload)
    console.log("gasEstimationResponse: ", gasEstimationResponse)
    if (gasEstimationResponse.isErr()) {
      console.error("[EVMTx] Gas estimation failed:", gasEstimationResponse.error)
      return
    }
    console.info("[EVMTx] Gas cost estimate:", gasEstimationResponse.value)

    // If only estimating gas, return now
    if (task.estimateGas) {
      console.info("[EVMTx] Task configured to only estimate gas; skipping transfer.")
      return
    }

    // Otherwise, perform the actual transfer
    const transferResp = await unionClient.transferAsset(transactionPayload)
    if (transferResp.isErr()) {
      console.error("[EVMTx] Transfer error:", transferResp.error)
      return
    }

    console.info("[EVMTx] Transfer success:", transferResp.value)
  } catch (error) {
    const msg = error instanceof Error ? error.message : String(error)
    console.error(`[EVMTx] Transfer exception: ${msg}`)
  }
}

/**
 * Main function that calls `checkPackets` repeatedly,
 * then does EVM transfers, in an infinite loop.
 */
export async function main() {
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
  console.info(`Using config file: ${configPath}`)

  // Load configuration
  const config = loadConfig(configPath)
  const chainPairs: ChainPair[] = config.interactions
  const oneHourMs = config.cycleIntervalMs

  // Optional array of EVM tasks
  const evmTransfers: EvmTransferConfig[] = config.evmTransfers ?? []

  while (true) {
    console.info("\n========== Starting IBC cross-chain checks ==========")
    for (const pair of chainPairs) {
      console.info(
        `Checking pair ${pair.sourceChain} <-> ${pair.destinationChain} with timeframe ${pair.timeframeMs}ms`
      )
      try {
        await checkPackets(pair.sourceChain, pair.destinationChain, pair.timeframeMs)
        console.info(`Check complete for pair ${pair.sourceChain} <-> ${pair.destinationChain}`)
      } catch (err) {
        console.error(
          `Error while checking pair ${pair.sourceChain} <-> ${pair.destinationChain}:`,
          err
        )
      }
    }

    // Optionally clear the reportedBlockHashes set every 3 cycles
    sleepCycleCount++
    if (sleepCycleCount % 3 === 0) {
      reportedBlockHashes.clear()
      console.info("Cleared reported block hashes.")
    }

    // Now do the EVM transfers for each config item:
    if (evmTransfers.length > 0) {
      console.info("\n========== Starting EVM transfer tasks ==========")
      for (const task of evmTransfers) {
        await doEvmTransfer(task)
      }
    }

    console.info(`\nAll checks & EVM tasks done. Sleeping for ${oneHourMs / 1000 / 60} minutes...`)
    await new Promise(resolve => setTimeout(resolve, oneHourMs))
  }
}

// Just call `main()` immediately
main().catch(err => console.error("Error in main()", err))
