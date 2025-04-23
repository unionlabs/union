import { Effect, Schedule, Data, Context, Logger } from "effect"
import { createPublicClient, http } from "viem"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import type { SigningCosmWasmClientOptions } from "@cosmjs/cosmwasm-stargate"
import { GasPrice } from "@cosmjs/stargate"
import { coins } from "@cosmjs/proto-signing"

import {
  channelBalance as EthereumChannelBalance,
  readErc20TotalSupply,
  ViemPublicClient as ViemPublicClientContext,
  ViemPublicClientDestination,
  EvmChannelDestination,
  readErc20Balance
} from "@unionlabs/sdk/evm"

import {
  channelBalance as CosmosChannelBalance,
  readCw20TotalSupply,
  createCosmWasmClient,
  CosmWasmClientContext,
  CosmWasmClientDestination,
  CosmosChannelDestination,
  createSigningCosmWasmClient
} from "@unionlabs/sdk/cosmos"

import yargs from "yargs"
import { hideBin } from "yargs/helpers"
import fs from "node:fs"
import { request, gql } from "graphql-request"
import Database from "better-sqlite3"
import fetch from "node-fetch"
import type { Database as BetterSqlite3Database } from "better-sqlite3"

// @ts-ignore
BigInt["prototype"].toJSON = function () {
  return this.toString()
}

type Hex = `0x${string}`
let db: BetterSqlite3Database

/**
 * Effect to trigger a BetterStack incident via the Uptime API
 */
export const triggerIncident = (
  summary: string,
  description: string,
  apiKey: string,
  requesterEmail: string,
  incidentName: string,
  teamName: string,
  isLocal: boolean
) =>
  isLocal
    ? Effect.sync(() => {
        console.info("Local mode: skipping triggerIncident")
        return { data: { id: "" } }
      })
    : Effect.tryPromise({
        try: () =>
          fetch("https://uptime.betterstack.com/api/v3/incidents", {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
              Authorization: `Bearer ${apiKey}`
            },
            body: JSON.stringify({
              summary,
              description,
              requester_email: requesterEmail,
              ...(teamName ? { team_name: teamName } : {}),
              call: false,
              sms: false,
              email: false,
              name: incidentName
            })
          }).then(async res => {
            const text = await res.text()
            if (!res.ok) throw new Error(`Trigger failed: ${text}`)
            return JSON.parse(text)
          }),
        catch: e => new Error(`Incident trigger error: ${e}`)
      })

/**
 * Effect to resolve an existing BetterStack incident via the Uptime API
 */
export const resolveIncident = (
  incidentId: string,
  apiKey: string,
  resolvedBy = "SENTINEL@union.build",
  isLocal: boolean
) =>
  isLocal
    ? Effect.sync(() => {
        console.info("Local mode: skipping resolveIncident")
        return { data: { id: incidentId } }
      })
    : Effect.tryPromise({
        try: () =>
          fetch(`https://uptime.betterstack.com/api/v3/incidents/${incidentId}/resolve`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
              Authorization: `Bearer ${apiKey}`
            },
            body: JSON.stringify({ resolved_by: resolvedBy })
          }).then(async res => {
            const text = await res.text()
            if (!res.ok) throw new Error(`Resolve failed: ${text}`)
            return JSON.parse(text)
          }),
        catch: e => new Error(`Incident resolve error: ${e}`)
      })

export function isFunded(db: BetterSqlite3Database, txHash: string) {
  const row = db.prepare(`SELECT 1 FROM funded_txs WHERE transaction_hash = ?`).get(txHash)
  return !!row
}

export function addFunded(db: BetterSqlite3Database, txHash: string) {
  db.prepare(`INSERT OR IGNORE INTO funded_txs (transaction_hash) VALUES (?)`).run(txHash)
}

export function getIncidentId(db: BetterSqlite3Database, packetHash: string): string | undefined {
  const row = db
    .prepare(`SELECT incident_id FROM transfer_errors WHERE packet_hash = ?`)
    .get(packetHash) as { incident_id: string } | undefined
  return row?.incident_id
}

export function markTransferError(
  db: BetterSqlite3Database,
  packetHash: string,
  incidentId: string
) {
  db.prepare(`
    INSERT OR REPLACE INTO transfer_errors
      (packet_hash, incident_id, inserted_at)
    VALUES (?, ?, strftime('%s','now')*1000)
  `).run(packetHash, incidentId)
}

export function clearTransferError(db: BetterSqlite3Database, packetHash: string) {
  db.prepare(`DELETE FROM transfer_errors WHERE packet_hash = ?`).run(packetHash)
}

export function hasErrorOpen(db: BetterSqlite3Database, packetHash: string) {
  const row = db.prepare(`SELECT 1 FROM transfer_errors WHERE packet_hash = ?`).get(packetHash)
  return !!row
}

function hexToUtf8(hex: string): string {
  // strip optional 0x
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex
  // build a Buffer from hex, then decode as UTF‑8
  return Buffer.from(clean, "hex").toString("utf8")
}

// Chain pair configuration
interface ChainPair {
  sourceChain: string
  destinationChain: string
  timeframeMs: number
  enabled: boolean
}

interface WrappedToken {
  chain: { universal_chain_id: string }
  denom: Hex
  wrapping: Array<{
    unwrapped_chain: { universal_chain_id: string }
    destination_channel_id: number
    unwrapped_denom: string
  }>
}

interface FundableAccounts {
  receiver_display: string
  traces: Array<{
    type: string
    transaction_hash: string
  }>
}

interface V2Channels {
  source_channel_id: string
}

interface ChannelInfo {
  source_channel_id: number
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
  packet_send_block_hash: string
  packet_hash: string
  timeout_timestamp: string
}

type ChainType = "evm" | "cosmos"

interface ChainConfigEntry {
  zkgmAddress: string
  rpc: string
  chainType: ChainType
  minter: string
}

type ChainConfig = Record<string, ChainConfigEntry>

// Combined configuration shape
interface ConfigFile {
  interactions: Array<ChainPair>
  cycleIntervalMs: number
  hasuraEndpoint: string
  chainConfig: ChainConfig
  signer_account_mnemonic: string
  betterstack_api_key: string
  db_path: string
  isLocal: boolean
}

class FilesystemError extends Data.TaggedError("FilesystemError")<{
  message: string
  cause: unknown
}> {}

export class Config extends Context.Tag("Config")<Config, { readonly config: ConfigFile }>() {}

const fetchWrappedTokens = (hasuraEndpoint: string) =>
  Effect.gen(function* () {
    const query = gql`
    query WrappedTokens {
      v2_tokens(where: { wrapping: { unwrapped_denom: { _is_null: false } } }) {
        chain { universal_chain_id }
        denom
        wrapping {
          unwrapped_chain { universal_chain_id }
          destination_channel_id
          unwrapped_denom
        }
      }
    }
  `

    const response: any = yield* Effect.tryPromise({
      try: () => request(hasuraEndpoint, query),
      catch: error => {
        console.error("fetchWrappedTokens failed:", error)
        throw error
      }
    })

    const tokens: WrappedToken[] = response?.v2_tokens || []
    return tokens
  })

const fetchFundableAccounts = (hasuraEndpoint: string) =>
  Effect.gen(function* () {
    const query = gql`
      query {
        v2_transfers(args: { p_destination_universal_chain_id: "babylon.bbn-1" }) {
          receiver_display
          traces {
            type
            transaction_hash
          }
        }
      }
    `

    const response: any = yield* Effect.tryPromise({
      try: () => request(hasuraEndpoint, query),
      catch: error => {
        console.error("fetchFundableAccounts failed:", error)
        throw error
      }
    })

    const tokens: FundableAccounts[] = response?.v2_transfers || []
    const filtered: FundableAccounts[] = tokens
      .map(({ receiver_display, traces }) => ({
        receiver_display,
        traces: traces
          .filter(
            trace =>
              trace.type === "WRITE_ACK" &&
              trace.transaction_hash != null &&
              !isFunded(db, trace.transaction_hash)
          )
          .map(trace => ({ type: trace.type, transaction_hash: trace.transaction_hash! }))
      }))
      .filter(acc => acc.traces.length > 0)

    return filtered
  })

const fetchSourceChannelId = (
  hasuraEndpoint: string,
  srcChain: string,
  dstChain: string,
  dstChannelId: number
) =>
  Effect.gen(function* () {
    const query = gql`
    query ChannelInfo($src: String!, $dst: String!, $dchan: Int!) {
      v2_channels(args: {
        p_source_universal_chain_id: $src,
        p_destination_universal_chain_id: $dst,
        p_destination_channel_id: $dchan
      }) {
        source_channel_id
      }
    }
  `

    const response: any = yield* Effect.tryPromise({
      try: () =>
        request(hasuraEndpoint, query, { src: srcChain, dst: dstChain, dchan: dstChannelId }),
      catch: error => {
        console.error("fetchSourceChannelId failed:", error)
        throw error
      }
    })

    const channels: ChannelInfo[] = response?.v2_channels || []
    return channels[0]?.source_channel_id
  })

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

const escrowSupplyControlLoop = Effect.repeat(
  Effect.gen(function* (_) {
    yield* Effect.log("Escrow supply control loop started")
    let config = (yield* Config).config

    const tokens = yield* fetchWrappedTokens(config.hasuraEndpoint)

    const evmChannelBalances = new Map<
      string, // chainId
      Map<string, bigint> // denom → balance
    >()
    const cosmosChannelBalances = new Map<string, Map<string, bigint>>()

    for (const token of tokens) {
      const srcChain = token.wrapping[0]?.unwrapped_chain.universal_chain_id
      const dstChain = token.chain.universal_chain_id

      const dstChannel = token.wrapping[0]?.destination_channel_id
      if (!srcChain || !dstChain || !dstChannel) {
        yield* Effect.log("Invalid token data. Skipping...")
        continue
      }
      const sourceChannelId = yield* fetchSourceChannelId(
        config.hasuraEndpoint,
        srcChain,
        dstChain,
        dstChannel
      )
      const srcCfg = config.chainConfig[srcChain]
      const dstCfg = config.chainConfig[dstChain]

      if (!srcCfg || !dstCfg) {
        yield* Effect.log("Invalid source or destination chain configuration. Skipping...")
        continue
      }

      if (!token.wrapping || token.wrapping.length === 0 || !token.wrapping[0]?.unwrapped_denom) {
        yield* Effect.log("No wrapping information available. Skipping...")
        continue
      }

      let srcChannelBal: bigint
      const key = token.wrapping[0]!.unwrapped_denom!
      const path = 0n

      if (srcCfg.chainType === "evm") {
        const client = createPublicClient({ transport: http(srcCfg.rpc) })
        srcChannelBal = yield* EthereumChannelBalance(path, key as Hex).pipe(
          Effect.provideService(ViemPublicClientDestination, { client }),
          Effect.provideService(EvmChannelDestination, {
            ucs03address: srcCfg.zkgmAddress as Hex,
            channelId: sourceChannelId!
          })
        )
        const chainMap = evmChannelBalances.get(srcChain) ?? new Map()
        const prev = chainMap.get(key) ?? 0n
        chainMap.set(key, prev + srcChannelBal)
        evmChannelBalances.set(srcChain, chainMap)
      } else {
        const client = yield* createCosmWasmClient(srcCfg.rpc)

        const srcChannelBalUnknown = yield* CosmosChannelBalance(path, hexToUtf8(key as Hex)).pipe(
          Effect.provideService(CosmWasmClientDestination, { client }),
          Effect.provideService(CosmosChannelDestination, {
            ucs03address: srcCfg.zkgmAddress,
            channelId: sourceChannelId!
          }),
          Effect.tapError(e => Effect.logError("Error fetching channel balance:", e))
        )
        srcChannelBal = BigInt(srcChannelBalUnknown as bigint)

        const chainMap = cosmosChannelBalances.get(srcChain) ?? new Map()
        const prev = chainMap.get(hexToUtf8(key as Hex)) ?? 0n
        chainMap.set(hexToUtf8(key as Hex), prev + srcChannelBal)
        cosmosChannelBalances.set(srcChain, chainMap)
      }

      let totalSupply = 0n
      if (dstCfg.chainType === "evm") {
        const client = createPublicClient({ transport: http(dstCfg.rpc) })
        totalSupply = yield* readErc20TotalSupply(token.denom).pipe(
          Effect.provideService(ViemPublicClientContext, { client })
        )
      } else {
        const client = yield* createCosmWasmClient(dstCfg.rpc)
        totalSupply = BigInt(
          yield* readCw20TotalSupply(hexToUtf8(token.denom)).pipe(
            Effect.provideService(CosmWasmClientContext, { client })
          )
        )
      }

      if (srcChannelBal < totalSupply) {
        const logEffect = Effect.annotateLogs({
          issueType: "TOTAL SUPPLY IS HIGHER THAN SOURCE CHANNEL BALANCE",
          sourceChain: `${srcChain}`,
          destinationChain: `${dstChain}`,
          denom: `${token.denom}`,
          unwrappedDenom: `${token.wrapping[0]?.unwrapped_denom}`,
          sourceChannelId: `${sourceChannelId}`,
          sourceChannelBal: `${srcChannelBal}`,
          totalSupply: `${totalSupply}`,
          destinationChannelId: `${dstChannel}`
        })(Effect.logError(`SUPPLY ERROR`))

        Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))
      } else {
        const logEffect = Effect.annotateLogs({
          sourceChain: `${srcChain}`,
          destinationChain: `${dstChain}`,
          denom: `${token.denom}`,
          unwrappedDenom: `${token.wrapping[0]?.unwrapped_denom}`,
          sourceChannelId: `${sourceChannelId}`,
          sourceChannelBal: `${srcChannelBal}`,
          totalSupply: `${totalSupply}`,
          destinationChannelId: `${dstChannel}`
        })(Effect.logInfo(`Channel balance is higher or equal, which is expected.`))

        Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))
      }
    }

    yield* Effect.log("Comparing aggregated channel balances to on‑chain holdings")

    for (const [chainId, { rpc, chainType, minter }] of Object.entries(config.chainConfig)) {
      if (chainType === "evm") {
        const client = createPublicClient({
          transport: http(rpc)
        })

        for (const [tokenAddr, channelSum] of evmChannelBalances.get(chainId) ?? []) {
          const onChain = yield* readErc20Balance(tokenAddr as Hex, minter as Hex).pipe(
            Effect.provideService(ViemPublicClientContext, { client }),
            Effect.tapError(e => Effect.logError("Error querying balanceOf:", e))
          )

          if (BigInt(onChain) < channelSum) {
            const errLog = Effect.annotateLogs({
              issueType: "AGGREGATE_GT_ONCHAIN",
              chainId,
              tokenAddr,
              minter,
              aggregated: channelSum.toString(),
              onChain: onChain.toString()
            })(Effect.logError("AGGREGATE_MISMATCH"))

            Effect.runFork(errLog.pipe(Effect.provide(Logger.json)))
          } else {
            const okLog = Effect.annotateLogs({
              chainId,
              tokenAddr,
              minter,
              aggregated: channelSum.toString(),
              onChain: onChain.toString()
            })(Effect.logInfo("AGGREGATE_OK"))

            Effect.runFork(okLog.pipe(Effect.provide(Logger.json)))
          }
        }
      } else {
        const cosmosClient = yield* createCosmWasmClient(rpc)

        for (const [denom, channelSum] of cosmosChannelBalances.get(chainId) ?? []) {
          const { amount } = yield* Effect.tryPromise({
            try: () => cosmosClient.getBalance(minter, denom),
            catch: e => new Error(`bank query failed: ${e}`)
          })

          if (BigInt(amount) < channelSum) {
            const errLog = Effect.annotateLogs({
              issueType: "AGGREGATE_GT_ONCHAIN",
              chainId,
              denom,
              minter,
              aggregated: channelSum.toString(),
              onChain: amount
            })(Effect.logError("AGGREGATE_MISMATCH"))

            Effect.runFork(errLog.pipe(Effect.provide(Logger.json)))
          } else {
            const okLog = Effect.annotateLogs({
              chainId,
              denom,
              minter,
              aggregated: channelSum.toString(),
              onChain: amount
            })(Effect.logInfo("AGGREGATE_OK"))

            Effect.runFork(okLog.pipe(Effect.provide(Logger.json)))
          }
        }
      }
    }
  }),
  Schedule.spaced("15 minutes")
)

const fundBabylonAccounts = Effect.repeat(
  Effect.gen(function* (_) {
    yield* Effect.log("Funding babylon accounts loop started")
    let config = (yield* Config).config

    const wallet = yield* Effect.tryPromise(() =>
      DirectSecp256k1HdWallet.fromMnemonic(config.signer_account_mnemonic, { prefix: "bbn" })
    )
    const options: SigningCosmWasmClientOptions = {
      gasPrice: GasPrice.fromString("0.025bbn")
    }
    const [senderAccount] = yield* Effect.tryPromise(() => wallet.getAccounts())

    const client = yield* createSigningCosmWasmClient(
      "https://rpc.bbn-1.babylon.chain.kitchen",
      wallet,
      options
    )

    if (!senderAccount || !senderAccount.address) {
      yield* Effect.logError("Sender account couldnt found!")
      return
    }
    const balance = yield* Effect.tryPromise(() => client.getBalance(senderAccount.address, "ubbn"))

    if (Number.parseInt(balance.amount) < 1_000_000) {
      const errLog = Effect.annotateLogs({
        issueType: "SPENDER_BALANCE_LOW",
        balance: balance.amount,
        chainId: "babylon.bbn-1",
        tokenAddr: "ubbn",
        account: senderAccount.address
      })(Effect.logError("SPENDER_BALANCE_LOW"))

      Effect.runFork(errLog.pipe(Effect.provide(Logger.json)))
      return
    }

    const fee = {
      amount: coins(500, "ubbn"),
      gas: "200000"
    }

    const accs = yield* fetchFundableAccounts(config.hasuraEndpoint)
    for (const acc of accs) {
      const receiver = acc.receiver_display
      const result = yield* Effect.tryPromise({
        try: () =>
          client.sendTokens(
            senderAccount.address,
            receiver,
            coins(10000, "ubbn"), // send 0.01 bbn
            fee
          ),
        catch: err => {
          console.error("raw sendTokens error:", err)
          throw err
        }
      })

      addFunded(db, result.transactionHash)

      const okLog = Effect.annotateLogs({
        sentAmount: "0.01",
        chainId: "babylon.bbn-1",
        tokenAddr: "ubbn",
        account: senderAccount.address,
        receiver,
        transactionHash: result.transactionHash
      })(Effect.logInfo("SENT_OK"))
      Effect.runFork(okLog.pipe(Effect.provide(Logger.json)))
    }
  }),
  Schedule.spaced("1 minutes")
)

const fetchOnlyUniBTC = (hasuraEndpoint: string) =>
  Effect.gen(function* () {
    let response: any

    // Next query: use the last sort_order as a cursor.
    const queryNext = gql`
        query MyQuery {
          v2_packets {
            decoded
          }
        }
      `
    response = yield* Effect.tryPromise({
      try: () => request(hasuraEndpoint, queryNext),
      catch: error => {
        console.error("Error in second query:", error)
        throw error
      }
    })

    for (const packet of response.v2_packets) {
      const operand = packet.decoded?.instruction?.operand
      if (!operand) continue
      if (operand.baseTokenName == "uniBTC" && operand.baseAmount) {
        const baseAmount = BigInt(operand.baseAmount)
        if (baseAmount >= 4000000n) {
          const logEffect = Effect.annotateLogs({
            packet: packet
          })(Effect.logError(`BIG_UNI_BTC`))
          Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))
        }
      }
    }
  })
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
const fetchPacketsUntilCutoff = (
  srcChain: string,
  dstChain: string,
  cutoffTimestamp: string,
  hasuraEndpoint: string
) =>
  Effect.gen(function* () {
    let allPackets: Packet[] = []
    let cursor: string | undefined
    let continueFetching = true

    while (continueFetching) {
      let response: any

      if (cursor) {
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
              packet_send_block_hash
              packet_hash
              timeout_timestamp
            }
          }
        `
        response = yield* Effect.tryPromise({
          try: () => request(hasuraEndpoint, queryNext, { sortOrder: cursor, srcChain, dstChain }),
          catch: error => {
            console.error("Error in second query:", error)
            throw error
          }
        })
      } else {
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
              packet_send_block_hash
              packet_hash
              timeout_timestamp
            }
          }
        `
        response = yield* Effect.tryPromise({
          try: () => request(hasuraEndpoint, queryFirst, { srcChain, dstChain }),
          catch: error => {
            console.info("Error in first query:", error)
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
export const checkPackets = (
  sourceChain: string,
  destinationChain: string,
  timeframeMs: number,
  hasuraEndpoint: string,
  betterstack_api_key: string,
  isLocal: boolean
) =>
  Effect.gen(function* () {
    const now = Date.now()
    const searchRangeMs = timeframeMs * 20
    const sinceDate = new Date(now - searchRangeMs).toISOString()

    yield* Effect.log(
      `Querying Hasura for packets >= ${sinceDate}, chain-pair: ${sourceChain} <-> ${destinationChain}`
    )

    const now_as_date = new Date(now).toISOString()
    yield* Effect.log(`now: ${now_as_date}`)

    yield* fetchOnlyUniBTC(hasuraEndpoint)

    const packets: Packet[] = yield* fetchPacketsUntilCutoff(
      sourceChain,
      destinationChain,
      sinceDate,
      hasuraEndpoint
    )
    yield* Effect.log(
      `Fetched ${packets.length} packets from Hasura from ${sourceChain} to ${destinationChain}`
    )
    // Process each packet.
    for (const p of packets) {
      if (!p.packet_send_timestamp) continue
      const sendTimeMs = new Date(p.packet_send_timestamp).getTime()
      // Only process packets that are older than the allowed timeframe.
      if (now - sendTimeMs < timeframeMs) continue

      if (now * 1000000 > BigInt(p.timeout_timestamp)) {
        continue
      }
      const sendTxHash = p.packet_send_transaction_hash ?? "?"
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
        const whole_description = {
          issueType: "RECV_MISSING",
          sendTxHash,
          sourceChain: `${sourceChain}`,
          destinationChain: `${destinationChain}`,
          explorerUrl: `https://btc.union.build/explorer/transfers/${sort_order_tx}`,
          minutesPassed: `${timeframeMs / 60 / 1000}`,
          packetSendBlockHash: p.packet_send_block_hash,
          packetHash: p.packet_hash,
          timeoutTimestamp: p.timeout_timestamp
        }
        const logEffect = Effect.annotateLogs(whole_description)(Effect.logError(`TRANSFER_ERROR`))

        if (!hasErrorOpen(db, p.packet_hash)) {
          const val = yield* triggerIncident(
            "TRANSFER_ERROR: " + `https://btc.union.build/explorer/transfers/${sort_order_tx}`,
            JSON.stringify(whole_description),
            betterstack_api_key,
            "SENTINEL@union.build",
            "TRANSFER_FAILED",
            "Union",
            isLocal
          )
          console.info("Incident triggered:", val)
          markTransferError(db, p.packet_hash, val.data.id)
        }

        Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))

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
        const whole_description = {
          issueType: "WRITE_ACK MISSING",
          sendTxHash,
          sourceChain: `${sourceChain}`,
          destinationChain: `${destinationChain}`,
          explorerUrl: `https://btc.union.build/explorer/transfers/${sort_order_tx}`,
          minutesPassed: `${timeframeMs / 60 / 1000}`,
          packetSendBlockHash: p.packet_send_block_hash,
          packetHash: p.packet_hash,
          timeoutTimestamp: p.timeout_timestamp
        }
        const logEffect = Effect.annotateLogs(whole_description)(Effect.logError(`TRANSFER_ERROR`))

        Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))

        if (!hasErrorOpen(db, p.packet_hash)) {
          const val = yield* triggerIncident(
            "TRANSFER_ERROR: " + `https://btc.union.build/explorer/transfers/${sort_order_tx}`,
            JSON.stringify(whole_description),
            betterstack_api_key,
            "SENTINEL@union.build",
            "TRANSFER_FAILED",
            "Union",
            isLocal
          )
          console.info("Incident triggered:", val)
          markTransferError(db, p.packet_hash, val.data.id)
        }

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
        const whole_description = {
          issueType: "ACK MISSING",
          sendTxHash,
          sourceChain: `${sourceChain}`,
          destinationChain: `${destinationChain}`,
          explorerUrl: `https://btc.union.build/explorer/transfers/${sort_order_tx}`,
          minutesPassed: `${timeframeMs / 60 / 1000}`,
          packetSendBlockHash: p.packet_send_block_hash,
          packetHash: p.packet_hash,
          timeoutTimestamp: p.timeout_timestamp
        }
        const logEffect = Effect.annotateLogs(whole_description)(Effect.logError(`TRANSFER_ERROR`))

        Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))

        if (!hasErrorOpen(db, p.packet_hash)) {
          const val = yield* triggerIncident(
            "TRANSFER_ERROR: " + `https://btc.union.build/explorer/transfers/${sort_order_tx}`,
            JSON.stringify(whole_description),
            betterstack_api_key,
            "SENTINEL@union.build",
            "TRANSFER_FAILED",
            "Union",
            isLocal
          )
          console.info("Incident triggered:", val)
          markTransferError(db, p.packet_hash, val.data.id)
        }

        continue
      }

      const incidentId = getIncidentId(db, p.packet_hash)
      if (incidentId) {
        console.info("Incident ID found:", incidentId)
        yield* resolveIncident(
          incidentId,
          betterstack_api_key,
          "Sentinel-Automatically resolved.",
          isLocal
        ),
          clearTransferError(db, p.packet_hash)
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
        pair.timeframeMs,
        config.hasuraEndpoint,
        config.betterstack_api_key,
        config.isLocal
      )
    }

    yield* Effect.log(
      `IBC Checks done (or skipped). Sleeping ${config.cycleIntervalMs / 1000 / 60} minutes...`
    )
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

  db = new Database(config.db_path)

  db.prepare(`
    CREATE TABLE IF NOT EXISTS funded_txs (
      transaction_hash TEXT PRIMARY KEY
    )
  `).run()

  db.prepare(`
    CREATE TABLE IF NOT EXISTS transfer_errors (
      packet_hash   TEXT PRIMARY KEY,
      incident_id   TEXT NOT NULL,
      inserted_at   INTEGER
    )
  `).run()

  console.info("Database opened at", config.db_path)

  yield* Effect.log("hasuraEndpoint: ", config.hasuraEndpoint)

  yield* Effect.all([runIbcChecksForever, escrowSupplyControlLoop, fundBabylonAccounts], {
    concurrency: "unbounded"
  }).pipe(Effect.provideService(Config, { config }))
})

Effect.runPromise(mainEffect).catch(err => Effect.logError("Error in mainEffect", err))
