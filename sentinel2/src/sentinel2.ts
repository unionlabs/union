import type { SigningCosmWasmClientOptions } from "@cosmjs/cosmwasm-stargate"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { coins } from "@cosmjs/proto-signing"
import { GasPrice } from "@cosmjs/stargate"
import {
  channelBalanceAtBlock as EthereumChannelBalanceAtBlock,
  EvmChannelDestination,
  readErc20BalanceAtBlock,
  readErc20TotalSupplyAtBlock,
  ViemPublicClient as ViemPublicClientContext,
  ViemPublicClientDestination,
} from "@unionlabs/sdk/evm"
import { Context, Data, Effect, Logger, Schedule } from "effect"
import { pipe } from "effect"
import * as Cause from "effect/Cause"
import tls from "node:tls"
import { createPublicClient, http } from "viem"

import {
  channelBalanceAtHeight as CosmosChannelBalanceAtHeight,
  CosmosChannelDestination,
  CosmWasmClientContext,
  createCosmWasmClient,
  createExtendedCosmWasmClient,
  createSigningCosmWasmClient,
  ExtendedCosmWasmClientContext,
  readCw20BalanceAtHeight,
  readCw20TokenInfo,
  readCw20TotalSupplyAtHeight,
} from "@unionlabs/sdk/cosmos"

import Database from "better-sqlite3"
import type { Database as BetterSqlite3Database } from "better-sqlite3"
import { gql, request } from "graphql-request"
import fetch from "node-fetch"
import fs from "node:fs"
import yargs from "yargs"
import { hideBin } from "yargs/helpers"

process.on("uncaughtException", err => {
  console.error("❌ Uncaught Exception:", err.stack || err)
})
process.on("unhandledRejection", (reason, promise) => {
  console.error("❌ Unhandled Rejection at:", promise, "reason:", reason)
})

// @ts-ignore
BigInt["prototype"].toJSON = function() {
  return this.toString()
}

/**
 * Checks whether a denom is a native token or CW20.
 * @param denom The denom address to check.
 * @returns An Effect that resolves to true if native, false if CW20.
 */
export const isDenomNative = (denom: string) =>
  Effect.gen(function*() {
    const client = (yield* CosmWasmClientContext).client

    return yield* readCw20TokenInfo(denom).pipe(
      Effect.provideService(CosmWasmClientContext, { client }),
      Effect.map(() => false), // If query succeeds => CW20 => false
      Effect.catchAllCause(() => Effect.succeed(true)), // If fails => native => true
    )
  })

/**
 * Checks whether the TLS cert for `host` is valid and
 * doesn’t expire within 1 week.
 *
 * @param host  hostname (e.g. "example.com")
 * @returns     Effect<never, Error, boolean>
 */
export const checkSslCertificate = (host: string, isExpiringInDays: number) =>
  Effect.tryPromise<boolean, Error>({
    try: () =>
      new Promise((resolve, reject) => {
        // connect on port 443, SNI = host
        const socket = tls.connect(443, host, { servername: host }, () => {
          const cert = socket.getPeerCertificate()
          socket.end()

          if (!cert || typeof cert.valid_to !== "string") {
            return reject(new Error("No certificate retrieved"))
          }

          const expiry = new Date(cert.valid_to)
          const oneWeekFromNow = Date.now() + isExpiringInDays * 24 * 60 * 60 * 1_000

          resolve(expiry.getTime() > oneWeekFromNow)
        })

        socket.on("error", reject)
      }),
    catch: e => new Error(`SSL certificate check failed: ${e}`),
  })
// helper to pull the cert expiry date out of a host:port
function getCertExpiry(endpoint: string): Promise<Date> {
  const { hostname, port } = new URL(endpoint)
  const portNum = port ? Number(port) : 443
  return new Promise((resolve, reject) => {
    const socket = tls.connect({ host: hostname, port: portNum, servername: hostname }, () => {
      const cert = socket.getPeerCertificate()
      socket.end()
      if (!cert || !cert.valid_to) {
        return reject(new Error(`no valid_to on cert for ${endpoint}`))
      }
      resolve(new Date(cert.valid_to))
    })
    socket.on("error", reject)
  })
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
  trigger_betterstack: boolean,
  requesterEmail: string,
  incidentName: string,
  teamName: string,
  isLocal: boolean,
) => {
  const remote = Effect.tryPromise<{ data: { id: string } }, Error>({
    try: () =>
      fetch("https://uptime.betterstack.com/api/v3/incidents", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${apiKey}`,
        },
        body: JSON.stringify({
          summary,
          description,
          requester_email: requesterEmail,
          ...(teamName ? { team_name: teamName } : {}),
          call: false,
          sms: false,
          email: false,
          name: incidentName,
        }),
      }).then(async res => {
        const text = await res.text()
        if (!res.ok) {
          throw new Error(`Trigger failed: ${text}`)
        }
        return JSON.parse(text)
      }),
    catch: e => new Error(`Incident trigger error: ${e}`),
  })
    // if anything went wrong, swallow it and return { data:{ id:"" } }
    .pipe(Effect.orElse(() => Effect.sync(() => ({ data: { id: "" } }))))

  if (isLocal) {
    return Effect.sync(() => {
      console.info("Local mode: skipping triggerIncident")
      return { data: { id: "" } }
    })
  }
  if (!trigger_betterstack) {
    return Effect.sync(() => {
      return { data: { id: "" } }
    })
  }
  return remote
}

/**
 * Effect to resolve an existing BetterStack incident via the Uptime API
 */
export const resolveIncident = (
  incidentId: string,
  apiKey: string,
  trigger_betterstack: boolean,
  isLocal: boolean,
  resolvedBy = "SENTINEL@union.build",
) => {
  if (!trigger_betterstack) {
    return Effect.sync(() => {
      return false
    })
  }
  if (isLocal) {
    return Effect.sync(() => {
      console.info("Local mode: skipping resolveIncident")
      return true
    })
  }

  return Effect.tryPromise<unknown, Error>({
    try: () =>
      fetch(`https://uptime.betterstack.com/api/v3/incidents/${incidentId}/resolve`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${apiKey}`,
        },
        body: JSON.stringify({ resolved_by: resolvedBy }),
      }).then(async res => {
        const text = await res.text()
        if (!res.ok) {
          throw new Error(`Resolve failed: ${text}`)
        }
        return JSON.parse(text)
      }),
    catch: e => new Error(`Incident resolve error: ${e}`),
  }).pipe(
    // if we parse successfully we consider it resolved
    Effect.map(() => true),
    // swallow any error and return `false`
    Effect.catchAllCause(err =>
      Effect.sync(() => {
        console.error("⚠️ resolveIncident failed:", err)
        return false
      })
    ),
  )
}

export function getAggregateIncident(db: BetterSqlite3Database, key: string): string | undefined {
  const row = db
    .prepare(`SELECT incident_id FROM aggregate_incidents WHERE key = ?`)
    .get(key) as { incident_id: string } | undefined
  return row?.incident_id
}

export function markAggregateIncident(db: BetterSqlite3Database, key: string, incidentId: string) {
  db.prepare(`
    INSERT OR REPLACE INTO aggregate_incidents
      (key, incident_id, inserted_at)
    VALUES (?, ?, strftime('%s','now')*1000)
  `).run(key, incidentId)
}

export function clearAggregateIncident(db: BetterSqlite3Database, key: string) {
  db.prepare(`DELETE FROM aggregate_incidents WHERE key = ?`).run(key)
}


export function getSupplyIncident(db: BetterSqlite3Database, key: string): string | undefined {
  const row = db.prepare(`SELECT incident_id FROM supply_incidents WHERE key = ?`).get(key) as
    | { incident_id: string }
    | undefined
  return row?.incident_id
}

export function markSupplyIncident(db: BetterSqlite3Database, key: string, incidentId: string) {
  db.prepare(`
    INSERT OR REPLACE INTO supply_incidents
      (key, incident_id, inserted_at)
    VALUES (?, ?, strftime('%s','now')*1000)
  `).run(key, incidentId)
}

export function clearSupplyIncident(db: BetterSqlite3Database, key: string) {
  db.prepare(`DELETE FROM supply_incidents WHERE key = ?`).run(key)
}

export function isFunded(db: BetterSqlite3Database, txHash: string) {
  const row = db.prepare(`SELECT 1 FROM funded_txs WHERE transaction_hash = ?`).get(txHash)
  return !!row
}

export function getSignerIncident(db: BetterSqlite3Database, key: string): string | undefined {
  const row = db.prepare(`SELECT incident_id FROM signer_incidents WHERE key = ?`).get(key) as
    | { incident_id: string }
    | undefined
  return row?.incident_id
}

export function markSignerIncident(db: BetterSqlite3Database, key: string, incidentId: string) {
  db.prepare(`
    INSERT OR REPLACE INTO signer_incidents
      (key, incident_id, inserted_at)
    VALUES (?, ?, strftime('%s','now')*1000)
  `).run(key, incidentId)
}

export function clearSignerIncident(db: BetterSqlite3Database, key: string) {
  db.prepare(`DELETE FROM signer_incidents WHERE key = ?`).run(key)
}
export function getSslIncident(db: BetterSqlite3Database, url: string): string | undefined {
  const row = db.prepare(`SELECT incident_id FROM ssl_incidents WHERE url = ?`).get(url) as
    | { incident_id: string }
    | undefined

  return row?.incident_id?.length ? row.incident_id : undefined
}

export function markSslIncident(db: BetterSqlite3Database, url: string, incidentId: string) {
  db.prepare(`
    INSERT OR REPLACE INTO ssl_incidents
      (url, incident_id, inserted_at)
    VALUES (?, ?, strftime('%s','now')*1000)
  `).run(url, incidentId)
}

export function clearSslIncident(db: BetterSqlite3Database, url: string) {
  db.prepare(`DELETE FROM ssl_incidents WHERE url = ?`).run(url)
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

export function hasErrorOpen(db: BetterSqlite3Database, sla: string, packetHash: string) {
  return !!db
    .prepare(
      `SELECT 1
         FROM transfer_errors
        WHERE sla = ?
          AND packet_hash = ?`,
    )
    .get(sla, packetHash)
}

export function markTransferError(
  db: BetterSqlite3Database,
  sla: string,
  packetHash: string,
  incidentId: string,
) {
  db.prepare(`
    INSERT OR REPLACE INTO transfer_errors
      (sla, packet_hash, incident_id, inserted_at)
    VALUES (?, ?, ?, strftime('%s','now')*1000)
  `).run(sla, packetHash, incidentId)
}

export function clearTransferError(db: BetterSqlite3Database, sla: string, packetHash: string) {
  db.prepare(`
    DELETE FROM transfer_errors
     WHERE sla = ?
       AND packet_hash = ?
  `).run(sla, packetHash)
}

export function getOpenErrors(
  db: BetterSqlite3Database,
  sla: string,
): Array<{ packet_hash: string; incident_id: string }> {
  return db
    .prepare(
      `SELECT packet_hash, incident_id
         FROM transfer_errors
        WHERE sla = ?`,
    )
    .all(sla) as Array<{ packet_hash: string; incident_id: string }>
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
  source_chain: {
    universal_chain_id: string
  }
  destination_chain: {
    universal_chain_id: string
  }
  packet_send_timestamp: string
  packet_hash: string
  status: string
  sort_order: string
}

type ChainType = "evm" | "cosmos"

interface ChainConfigEntry {
  zkgmAddress: string
  rpc: string
  restUrl: string
  chainType: ChainType
  minter: string
}

type ChainConfig = Record<string, ChainConfigEntry>

export interface SignerBalanceThresholds {
  [plugin: string]: bigint
}

export type PortSignerBalances = Record<string, SignerBalanceThresholds>

export type SignerBalancesConfig = Record<string, PortSignerBalances>

// Combined configuration shape
interface ConfigFile {
  cycleIntervalMs: number
  hasuraEndpoint: string
  rpcHostEndpoints: string[]
  signerBalances: SignerBalancesConfig
  chainConfig: ChainConfig
  signer_account_mnemonic: string
  betterstack_api_key: string
  trigger_betterstack: boolean
  dbPath: string
  isLocal: boolean
}

class FilesystemError extends Data.TaggedError("FilesystemError")<{
  message: string
  cause: unknown
}> {}

export class Config extends Context.Tag("Config")<Config, { readonly config: ConfigFile }>() {}

const fetchWrappedTokens = (hasuraEndpoint: string) =>
  Effect.gen(function*() {
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
      },
    })

    const tokens: Array<WrappedToken> = response?.v2_tokens || []
    return tokens
  })

const fetchFundableAccounts = (hasuraEndpoint: string) =>
  Effect.gen(function*() {
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
      },
    })

    const tokens: Array<FundableAccounts> = response?.v2_transfers || []
    const filtered: Array<FundableAccounts> = tokens
      .map(({ receiver_display, traces }) => ({
        receiver_display,
        traces: traces
          .filter(
            trace =>
              trace.type === "WRITE_ACK"
              && trace.transaction_hash != null
              && !isFunded(db, trace.transaction_hash),
          )
          // biome-ignore lint/style/noNonNullAssertion: <explanation>
          .map(trace => ({ type: trace.type, transaction_hash: trace.transaction_hash! })),
      }))
      .filter(acc => acc.traces.length > 0)

    return filtered
  })

const fetchSourceChannelId = (
  hasuraEndpoint: string,
  srcChain: string,
  dstChain: string,
  dstChannelId: number,
) =>
  Effect.gen(function*() {
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
      },
    })

    const channels: Array<ChannelInfo> = response?.v2_channels || []
    return channels[0]?.source_channel_id
  })

function loadConfig(configPath: string) {
  return Effect.tryPromise({
    // biome-ignore lint/suspicious/useAwait: <explanation>
    try: async () => {
      if (!fs.existsSync(configPath)) {
        throw new Error("Config file not found. Ensure config.json exists.")
      }
      const rawData = fs.readFileSync(configPath, "utf-8")
      const config: ConfigFile = JSON.parse(rawData)

      return config
    },
    catch: error =>
      new FilesystemError({
        message: "Config file is invalid.",
        cause: error,
      }),
  })
}
const escrowSupplyControlLoop = Effect.repeat(
  Effect.gen(function*(_) {
    yield* Effect.log("Escrow supply control loop started")
    let config = (yield* Config).config

    const tokens = yield* fetchWrappedTokens(config.hasuraEndpoint)

    const evmChannelBalances = new Map<
      string, // chainId
      Map<string, bigint> // denom → balance
    >()
    const blockNumbers = new Map<string, bigint>()
    const cosmosChannelBalances = new Map<string, Map<string, bigint>>()
    if (tokens) {
      for (const { rpc, chainType } of Object.values(config.chainConfig)) {
        if (chainType === "evm") {
          const latest = yield* Effect.tryPromise({
            try: () => {
              const client = createPublicClient({ transport: http(rpc) })
              return client.getBlockNumber()
            },
            catch: e => new Error(`Failed to fetch blockNumber for ${rpc}: ${String(e)}`),
          })
          console.info("latest blockNumber", latest, "rpc", rpc)
          blockNumbers.set(rpc, BigInt(latest))
        } else {
          const client = yield* createCosmWasmClient(rpc)
          const latest = yield* Effect.tryPromise({
            try: () => {
              return client.getHeight()
            },
            catch: e => new Error(`Failed to fetch blockNumber for ${rpc}: ${String(e)}`),
          })
          console.info("latest blockNumber", latest, "rpc", rpc)
          blockNumbers.set(rpc, BigInt(latest))
        }
      }
      yield* Effect.log("Fetched wrapped tokens length:", tokens.length)
      for (const token of tokens) {
        const srcChain = token.wrapping[0]?.unwrapped_chain.universal_chain_id
        const dstChain = token.chain.universal_chain_id

        const dstChannel = token.wrapping[0]?.destination_channel_id
        // biome-ignore lint/complexity/useSimplifiedLogicExpression: <explanation>
        if (!srcChain || !dstChain || !dstChannel) {
          yield* Effect.log("Invalid token data. Skipping...")
          continue
        }
        const sourceChannelId = yield* fetchSourceChannelId(
          config.hasuraEndpoint,
          srcChain,
          dstChain,
          dstChannel,
        )
        if (!sourceChannelId) {
          yield* Effect.log("No source channel ID found. Skipping...")
          continue
        }

        const srcCfg = config.chainConfig[srcChain]
        const dstCfg = config.chainConfig[dstChain]

        if (!(srcCfg && dstCfg)) {
          yield* Effect.log(
            "Invalid source or destination chain configuration. Skipping... srcChain:",
            srcChain,
            "dstChain:",
            dstChain,
          )
          continue
        }

        if (!token.wrapping || token.wrapping.length === 0 || !token.wrapping[0]?.unwrapped_denom) {
          yield* Effect.log("No wrapping information available. Skipping...")
          continue
        }

        let srcChannelBal: bigint
        // biome-ignore lint/style/noNonNullAssertion: <explanation>
        const key = token.wrapping[0]!.unwrapped_denom!
        const path = 0n

        if (srcCfg.chainType === "evm") {
          const client = createPublicClient({ transport: http(srcCfg.rpc) })
          const evmHeight = blockNumbers.get(srcCfg.rpc)!
          if (!evmHeight) {
            yield* Effect.log("No block number found for source chain:", srcChain)
            continue
          }
          const srcChannelBalHere = yield* EthereumChannelBalanceAtBlock(
            path,
            key as Hex,
            evmHeight,
          ).pipe(
            Effect.provideService(ViemPublicClientDestination, { client }),
            Effect.provideService(EvmChannelDestination, {
              ucs03address: srcCfg.zkgmAddress as Hex,
              // biome-ignore lint/style/noNonNullAssertion: <explanation>
              channelId: sourceChannelId!,
            }),
            Effect.catchAllCause((cause) => {
              console.error(`Error fetching channel balance: ${Cause.pretty(cause)}`);
              return Effect.succeed(null); 
            })
          )
          if (!srcChannelBalHere) {
            yield* Effect.log("No srcChannelBal for token:", token.denom)
            continue
          }
          srcChannelBal = BigInt(srcChannelBalHere as bigint)
          const chainMap = evmChannelBalances.get(srcChain) ?? new Map()
          const prev = chainMap.get(key) ?? 0n
          chainMap.set(key, prev + srcChannelBal)
          evmChannelBalances.set(srcChain, chainMap)
        } else {
          const client = yield* createCosmWasmClient(srcCfg.rpc)
          const extClient = yield* createExtendedCosmWasmClient(srcCfg.rpc, srcCfg.restUrl)

          const cosmosHeight = blockNumbers.get(srcCfg.rpc)!
          if (!cosmosHeight) {
            yield* Effect.log("No block number found for cosmos - source chain:", srcChain)
            continue
          }

          const srcChannelBalUnknown = yield* CosmosChannelBalanceAtHeight(
            path,
            hexToUtf8(key as Hex),
            Number(cosmosHeight),
          ).pipe(
            Effect.provideService(ExtendedCosmWasmClientContext, { client: extClient }),
            Effect.provideService(CosmosChannelDestination, {
              ucs03address: srcCfg.zkgmAddress,
              // biome-ignore lint/style/noNonNullAssertion: <explanation>
              channelId: sourceChannelId!,
            }),
            Effect.catchAllCause((cause) => {
              console.error(`Error fetching channel balance: ${Cause.pretty(cause)}`);
              return Effect.succeed(null); 
            })
          )
          if (!srcChannelBalUnknown) {
            yield* Effect.log("No srcChannelBalUnknown for token:", token.denom)
            continue
          }
          srcChannelBal = BigInt(srcChannelBalUnknown as bigint)

          const chainMap = cosmosChannelBalances.get(srcChain) ?? new Map()
          const prev = chainMap.get(hexToUtf8(key as Hex)) ?? 0n
          chainMap.set(hexToUtf8(key as Hex), prev + srcChannelBal)
          cosmosChannelBalances.set(srcChain, chainMap)
        }

        let totalSupply = 0n
        if (dstCfg.chainType === "evm") {
          const client = createPublicClient({ transport: http(dstCfg.rpc) })
          const evmHeight = blockNumbers.get(dstCfg.rpc)!
          if (!evmHeight) {
            yield* Effect.log("No block number found for destination chain:", dstChain)
            continue
          }
          const totalSupplyHere = yield* readErc20TotalSupplyAtBlock(token.denom, evmHeight).pipe(
            Effect.provideService(ViemPublicClientContext, { client }),
            Effect.catchAllCause((cause) => {
              console.error(`Failed to fetch total supply for token ${token.denom}: ${Cause.pretty(cause)}`);
              return Effect.succeed(null); 
            })
          );
        
          if (!totalSupplyHere) {
            yield* Effect.log("No total supply found for token:", token.denom)
            continue
          }
          totalSupply = BigInt(totalSupplyHere as bigint)
        } else {
          const extClient = yield* createExtendedCosmWasmClient(dstCfg.rpc, dstCfg.restUrl)

          const cosmosHeight = blockNumbers.get(dstCfg.rpc)!
          if (!cosmosHeight) {
            yield* Effect.log("No block number found for cosmos - destination chain:", dstChain)
            continue
          }

          const totalSupplyHere = yield* readCw20TotalSupplyAtHeight(
            hexToUtf8(token.denom),
            Number(cosmosHeight),
          ).pipe(
            Effect.provideService(ExtendedCosmWasmClientContext, { client: extClient }),
            Effect.catchAllCause((cause) => {
              console.error(`Error fetching total supply: ${Cause.pretty(cause)}`);
              return Effect.succeed(null); 
            })
          )
          if (!totalSupplyHere) {
            yield* Effect.log("No total supply found for token:", token.denom)
            continue
          }
          totalSupply = BigInt(totalSupplyHere)
        }
        
        const supplyKey = `${srcChain}:${dstChain}:${token.denom}`
        const existingSupplyIncident = getSupplyIncident(db, supplyKey)

        if (srcChannelBal < totalSupply) {
          if (!existingSupplyIncident) {
            const inc = yield* triggerIncident(
              `SUPPLY_ERROR @ ${supplyKey}`,
              JSON.stringify({
                issueType: "TOTAL_SUPPLY_GT_CHANNEL_BALANCE",
                sourceChain: srcChain,
                destinationChain: dstChain,
                denom: token.denom,
                unwrappedDenom: token.wrapping[0]?.unwrapped_denom,
                sourceChannelId,
                sourceChannelBal: srcChannelBal.toString(),
                totalSupply: totalSupply.toString(),
              }),
              config.betterstack_api_key,
              config.trigger_betterstack,
              "SENTINEL@union.build",
              "TOTAL_SUPPLY_GT_CHANNEL_BALANCE",
              "Union",
              config.isLocal,
            )
            if (inc.data.id) markSupplyIncident(db, supplyKey, inc.data.id)
          }        

          const logEffect = Effect.annotateLogs({
            issueType: "TOTAL SUPPLY IS HIGHER THAN SOURCE CHANNEL BALANCE",
            sourceChain: `${srcChain}`,
            destinationChain: `${dstChain}`,
            denom: `${token.denom}`,
            unwrappedDenom: `${token.wrapping[0]?.unwrapped_denom}`,
            sourceChannelId: `${sourceChannelId}`,
            sourceChannelBal: `${srcChannelBal}`,
            totalSupply: `${totalSupply}`,
            destinationChannelId: `${dstChannel}`,
          })(Effect.logError(`SUPPLY_ERROR`))

          Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))
        } else {
          if (existingSupplyIncident) {
            const didResolve = yield* resolveIncident(
              existingSupplyIncident,
              config.betterstack_api_key,
              config.trigger_betterstack,
              config.isLocal,
              "Sentinel: supply back in sync",
            )
            if (didResolve) clearSupplyIncident(db, supplyKey)
            }
        
          const logEffect = Effect.annotateLogs({
            sourceChain: `${srcChain}`,
            destinationChain: `${dstChain}`,
            denom: `${token.denom}`,
            unwrappedDenom: `${token.wrapping[0]?.unwrapped_denom}`,
            sourceChannelId: `${sourceChannelId}`,
            sourceChannelBal: `${srcChannelBal}`,
            totalSupply: `${totalSupply}`,
            destinationChannelId: `${dstChannel}`,
          })(Effect.logInfo(`Channel balance is higher or equal, which is expected.`))

          Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))
        }
      }

      yield* Effect.log("Comparing aggregated channel balances to on‑chain holdings")

      for (
        const [chainId, { rpc, restUrl, chainType, minter }] of Object.entries(
          config.chainConfig,
        )
      ) {
        if (chainType === "evm") {
          const client = createPublicClient({
            transport: http(rpc),
          })

          const evmHeight = blockNumbers.get(rpc)!
          if (!evmHeight) {
            yield* Effect.log("No block number found for source chain:", chainId)
            continue
          }

          for (const [tokenAddr, channelSum] of evmChannelBalances.get(chainId) ?? []) {
            const onChainRaw = yield* readErc20BalanceAtBlock(
              tokenAddr as Hex,
              minter as Hex,
              evmHeight,
            ).pipe(
              Effect.provideService(ViemPublicClientContext, { client }),
              Effect.catchAllCause((cause) => {
                console.error(`Error querying balanceOf: ${Cause.pretty(cause)}`);
                return Effect.succeed(null); 
              })
            )
            if (!onChainRaw) {
              yield* Effect.log("No balance found for denom:", tokenAddr)
              continue
            }
            const onChain = BigInt(onChainRaw as bigint)
            const aggregateKey = `${chainId}:${tokenAddr}`
            const existingAgg = getAggregateIncident(db, aggregateKey)


            if (onChain < channelSum) {
              if (!existingAgg) {
                const inc = yield* triggerIncident(
                  `AGGREGATE_MISMATCH @ ${aggregateKey}`,
                  JSON.stringify({
                    issueType: "AGGREGATE_GT_ONCHAIN",
                    chainId,
                    tokenAddr,
                    minter,
                    aggregated: channelSum.toString(),
                    onChain: onChain.toString(),
                  }),
                  config.betterstack_api_key,
                  config.trigger_betterstack,
                  "SENTINEL@union.build",
                  "AGGREGATE_GT_ONCHAIN",
                  "Union",
                  config.isLocal,
                )
                if (inc.data.id) markAggregateIncident(db, aggregateKey, inc.data.id)
              }

              const errLog = Effect.annotateLogs({
                issueType: "AGGREGATE_GT_ONCHAIN",
                chainId,
                tokenAddr,
                minter,
                aggregated: channelSum.toString(),
                onChain: onChain.toString(),
              })(Effect.logError("AGGREGATE_MISMATCH"))

              Effect.runFork(errLog.pipe(Effect.provide(Logger.json)))
            } else {
              if (existingAgg) {
                const didResolve = yield* resolveIncident(
                  existingAgg,
                  config.betterstack_api_key,
                  config.trigger_betterstack,
                  config.isLocal,
                  "Sentinel: aggregate back in sync",
                )
                if (didResolve) clearAggregateIncident(db, aggregateKey)
              }
          
              const okLog = Effect.annotateLogs({
                chainId,
                tokenAddr,
                minter,
                aggregated: channelSum.toString(),
                onChain: onChain.toString(),
              })(Effect.logInfo("AGGREGATE_OK"))

              Effect.runFork(okLog.pipe(Effect.provide(Logger.json)))
            }
          }
        } else {
          const cosmosClient = yield* createCosmWasmClient(rpc)
          const extClient = yield* createExtendedCosmWasmClient(rpc, restUrl)

          for (const [denom, channelSum] of cosmosChannelBalances.get(chainId) ?? []) {
            const isDenomNativeHere = yield* isDenomNative(denom).pipe(
              Effect.provideService(CosmWasmClientContext, { client: cosmosClient }),
              Effect.catchAllCause((cause) => {
                console.error(`Error checking denom type: ${Cause.pretty(cause)}`);
                return Effect.succeed(null); 
              })
            )
            let amount
            const cosmosHeight = blockNumbers.get(rpc)!
            if (!cosmosHeight) {
              yield* Effect.log("No block number found for cosmos - chain:", chainId)
              continue
            }
            if (isDenomNativeHere) {
              // const balance = yield* Effect.tryPromise({
              //   try: () => cosmosClient.getBalance(minter, denom),
              //   catch: e => new Error(`bank query failed: ${e}`),
              // })
              const balance = yield* Effect.tryPromise({
                try: () => extClient.getBalanceAtHeight(minter, denom, Number(cosmosHeight)),
                catch: e => new Error(`bank query failed: ${e}`),
              })
              if (!balance) {
                yield* Effect.log("No balance found for denom:", denom)
                continue
              }
              amount = BigInt(balance)
            } else {


              const balance = yield* readCw20BalanceAtHeight(
                denom,
                minter,
                Number(cosmosHeight),
              ).pipe(
                Effect.provideService(ExtendedCosmWasmClientContext, { client: extClient }),
                Effect.catchAllCause((cause) => {
                  console.error(`Error fetching balance: ${Cause.pretty(cause)}`);
                  return Effect.succeed(null); 
                })
              )
              if (!balance) {
                yield* Effect.log("No balance found for denom:", denom)
                continue
              }
              amount = BigInt(balance)
            }
            if (BigInt(amount) < channelSum) {
              const errLog = Effect.annotateLogs({
                issueType: "AGGREGATE_GT_ONCHAIN",
                chainId,
                denom,
                minter,
                aggregated: channelSum.toString(),
                onChain: amount,
              })(Effect.logError("AGGREGATE_MISMATCH"))

              Effect.runFork(errLog.pipe(Effect.provide(Logger.json)))
            } else {
              const okLog = Effect.annotateLogs({
                chainId,
                denom,
                minter,
                aggregated: channelSum.toString(),
                onChain: amount,
              })(Effect.logInfo("AGGREGATE_OK"))

              Effect.runFork(okLog.pipe(Effect.provide(Logger.json)))
            }
          }
        }
      }
    }
  }).pipe(
    Effect.catchAllCause(err =>
      Effect.sync(() => {
        console.error("⚠️ escrowSupplyControlLoop iteration failed, skipping:", err)
      })
    ),
  ),
  Schedule.spaced("1 hours"),
)

const fundBabylonAccounts = Effect.repeat(
  Effect.gen(function*(_) {
    yield* Effect.log("Funding babylon accounts loop started")
    let config = (yield* Config).config
    if (config.isLocal) {
      yield* Effect.log("Local mode: skipping funding babylon accounts")
      return
    }

    const wallet = yield* Effect.tryPromise(() =>
      DirectSecp256k1HdWallet.fromMnemonic(config.signer_account_mnemonic, { prefix: "bbn" })
    )
    const options: SigningCosmWasmClientOptions = {
      gasPrice: GasPrice.fromString("0.025bbn"),
    }
    const [senderAccount] = yield* Effect.tryPromise(() => wallet.getAccounts())

    const client = yield* createSigningCosmWasmClient(
      "https://rpc.bbn-1.babylon.chain.kitchen",
      wallet,
      options,
    )

    if (!senderAccount?.address) {
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
        account: senderAccount.address,
      })(Effect.logError("SPENDER_BALANCE_LOW"))

      Effect.runFork(errLog.pipe(Effect.provide(Logger.json)))
      return
    }

    const fee = {
      amount: coins(500, "ubbn"),
      gas: "200000",
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
            fee,
          ),
        catch: err => {
          console.error("raw sendTokens error:", err)
          throw err
        },
      })

      addFunded(db, result.transactionHash)

      const okLog = Effect.annotateLogs({
        sentAmount: "0.01",
        chainId: "babylon.bbn-1",
        tokenAddr: "ubbn",
        account: senderAccount.address,
        receiver,
        transactionHash: result.transactionHash,
      })(Effect.logInfo("SENT_OK"))
      Effect.runFork(okLog.pipe(Effect.provide(Logger.json)))
    }
  }),
  Schedule.spaced("1 minutes"),
)

interface PostRequestInput {
  url: string
  port?: number
  headers: Record<string, string>
  payload: unknown
}

interface PostRequestError {
  readonly _tag: "PostRequestError"
  readonly message: string
  readonly status?: number
}
interface GetRequestError {
  readonly _tag: "GetRequestError"
  readonly message: string
  readonly status?: number
}

// 1) make headers always defined by giving it a default
export const safeGetRequest = ({
  url,
  port,
  headers = {} as Record<string, string>,
}: {
  url: string
  port?: number
  headers?: Record<string, string>
}) =>
  Effect.tryPromise({
    try: async () => {
      const fullUrl = port ? `${url}:${port}` : url
      const res = await fetch(fullUrl, { method: "GET", headers }) // headers is now always a Record<string,string>
      const text = await res.text()
      if (!res.ok) {
        throw { _tag: "GetRequestError", message: `GET ${res.status}`, status: res.status }
      }
      return text
    },
    catch: error =>
      ({
        _tag: "GetRequestError",
        message: error instanceof Error ? error.message : String(error),
        status: (error as any)?.status,
      }) as GetRequestError,
  })

export const safePostRequest = ({ url, port, headers, payload }: PostRequestInput) => {
  const fullUrl = port ? `${url}:${port}` : url

  return Effect.tryPromise({
    try: () =>
      fetch(fullUrl, {
        method: "POST",
        headers,
        body: JSON.stringify(payload),
      }).then(async response => {
        if (response.status === 200) {
          return await response.json()
        }
        const text = await response.text().catch(() => "")
        // biome-ignore lint/style/useThrowOnlyError: <explanation>
        throw {
          _tag: "PostRequestError",
          message: `Non-200 status: ${response.status} body: ${text}`,
          status: response.status,
        }
      }),
    catch: error =>
      ({
        _tag: "PostRequestError",
        message: error instanceof Error
          ? error.message
          : typeof error === "object"
          ? JSON.stringify(error)
          : String(error),
        status: (error as any)?.status,
      }) satisfies PostRequestError,
  })
}

export const checkSSLCertificates = Effect.repeat(
  Effect.gen(function*(_) {
    yield* Effect.log("Spawning checkSSLCertificates loop")
    const { config } = yield* Config
    const rpchostEndpoints = config.rpcHostEndpoints

    for (const rpchostEndpoint of rpchostEndpoints) {
      const pageHtml: string = yield* safeGetRequest({
        url: rpchostEndpoint,
        port: 443,
        headers: {},
      }).pipe(Effect.retry(Schedule.spaced("2 minutes")))
      const endpointAnchorRegex = /<a\s+href="([^"]+)">https?:\/\/[^<]+<\/a>/gi
      const links: string[] = []
      let m: RegExpExecArray | null
      while ((m = endpointAnchorRegex.exec(pageHtml))) {
        const href = m[1]
        if (href) {
          links.push(href)
        }
      }
      const uniqueEndpoints = Array.from(new Set(links))
      yield* Effect.log(`Found ${uniqueEndpoints.length} endpoints}`)

      const now = Date.now()
      const fourDaysMs = 4 * 24 * 60 * 60 * 1000
      for (const url of uniqueEndpoints) {
        const existingIncident = getSslIncident(db, url)
        const expiry: Date = yield* Effect.tryPromise({
          try: () => getCertExpiry(url),
          catch: e => new Error(`SSL check failed for ${url}: ${String(e)}`),
        })

        const msLeft = expiry.getTime() - now
        const description = JSON.stringify({ endpoint: url, expiresAt: expiry.toISOString() })

        if (msLeft <= fourDaysMs) {
          if (!existingIncident) {
            const inc = yield* triggerIncident(
              `SSL expiring soon @ ${url}`,
              description,
              config.betterstack_api_key,
              config.trigger_betterstack,
              "SENTINEL@union.build",
              "SSL_CERT_EXPIRY",
              "Union",
              config.isLocal,
            )
            if (inc.data.id) {
              markSslIncident(db, url, inc.data.id)
            }
          }
          yield* Effect.logError(`SSL expiring in ${(msLeft / 86400000).toFixed(1)} day. @ ${url}`)
        } else {
          if (existingIncident) {
            const didResolve = yield* resolveIncident(
              existingIncident,
              config.betterstack_api_key,
              config.trigger_betterstack,
              config.isLocal,
              "Sentinel: SSL renewed",
            )
            if (didResolve) {
              clearSslIncident(db, url)
            }
          }
          yield* Effect.log(`SSL ok @ ${url}, expires in ${(msLeft / 86400000).toFixed(1)} days`)
        }
      }
    }
  }),
  Schedule.spaced("6 hours"),
)

export const checkBalances = Effect.repeat(
  Effect.gen(function*(_) {
    yield* Effect.log("Spawning per-plugin balance checks…")
    const { config } = yield* Config
    const sbConfig = config.signerBalances

    for (const [url, ports] of Object.entries(sbConfig)) {
      for (const [portStr, plugins] of Object.entries(ports)) {
        const port = Number(portStr)

        const portKey = `${url}:${port}`
        const existingPortIncident = getSignerIncident(db, portKey)

        const [probeJson, durationMs] = yield* Effect.gen(function*($) {
          const start = Date.now()
          const resp = yield* Effect.tryPromise<Response, Error>({
            try: () =>
              (fetch(`${url}:${port}`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
              }) as unknown) as PromiseLike<Response>,
            catch: e => new Error(`RPC probe connection failed: ${e}`),
          })
          
          const text = yield* Effect.tryPromise<string, Error>({
            try: () => resp.text(),
            catch: e => new Error(`RPC probe read failed: ${e}`),
          })
          const took = Date.now() - start

          let json: any = null
          try { json = JSON.parse(text) } catch { /* leave json=null */ }

          return [json, took] as const
        })

        if (!probeJson || typeof probeJson.error !== "object") {
          yield* Effect.logError(`SIGNER_BALANCE_PORT_DOWN @ ${portKey}`)
          if (!existingPortIncident) {
            const inc = yield* triggerIncident(
              `SIGNER_BALANCE_PORT_DOWN @ ${portKey}`,
              `no RPC response from ${url}:${port}`,
              config.betterstack_api_key,
              config.trigger_betterstack,
              "SENTINEL@union.build",
              "SIGNER_BALANCE_PORT_DOWN",
              "Union",
              config.isLocal,
            )
            markSignerIncident(db, portKey, inc.data.id)
          }
          continue
        }

        const errMsg = String(probeJson.error.message)
        if (errMsg !== "Parse error") {
          yield* Effect.logError(`SIGNER_BALANCE_RPC_ERROR @ ${portKey}: ${errMsg}`)
          if (!existingPortIncident) {
            const inc = yield* triggerIncident(
              `SIGNER_BALANCE_RPC_ERROR @ ${portKey}`,
              `unexpected RPC error: ${errMsg}`,
              config.betterstack_api_key,
              config.trigger_betterstack,
              "SENTINEL@union.build",
              "SIGNER_BALANCE_RPC_ERROR",
              "Union",
              config.isLocal,
            )
            markSignerIncident(db, portKey, inc.data.id)
          }
          continue
        }

        yield* Effect.log(`SIGNER_BALANCE_RPC_OK @ ${portKey} in ${durationMs}ms`)
        if (existingPortIncident) {
          const resolved = yield* resolveIncident(
            existingPortIncident,
            config.betterstack_api_key,
            config.trigger_betterstack,
            config.isLocal,
            "Sentinel: RPC back online",
          )
          if (resolved) clearSignerIncident(db, portKey)
        }


        for (const [plugin, expectedThreshold] of Object.entries(plugins)) {
          const payload = [
            {
              jsonrpc: "2.0",
              id: 1,
              method: "voyager_pluginCustom",
              params: [plugin, "signerBalances", []] as const,
            },
          ]

          const callWithRetry = safePostRequest({
            url,
            port,
            headers: { "Content-Type": "application/json" },
            payload,
          })

          const worker = Effect.gen(function*(_) {
            const result = yield* callWithRetry
            if (result) {
              if (!Array.isArray(result) || result.length === 0) {
                yield* Effect.logError(`Unexpected response shape for ${plugin} @ ${url}:${port}`)
                return
              }

              const rpcObj = (result[0] as any).result
              if (typeof rpcObj !== "object" || rpcObj === null) {
                yield* Effect.logError(`No 'result' object for ${plugin} @ ${url}:${port}`)
                return
              }

              for (const [wallet, balStr] of Object.entries(rpcObj)) {
                let bal = BigInt(balStr as string)

                const tags = {
                  plugin,
                  url,
                  port: portStr,
                  wallet,
                  balance: bal.toString(),
                  expected: expectedThreshold.toString(),
                }

                const key = `${url}:${port}:${plugin}:${wallet}`
                const existing = getSignerIncident(db, key)

                if (bal < expectedThreshold) {
                  const logEffect = Effect.annotateLogs(tags)(Effect.logError("SIGNER_BALANCE_LOW"))
                  Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))

                  if (!existing) {
                    const inc = yield* triggerIncident(
                      `SIGNER_BALANCE_LOW @ ${key}`,
                      JSON.stringify({
                        plugin,
                        url,
                        port: portStr,
                        wallet,
                        balance: bal.toString(),
                      }),
                      config.betterstack_api_key,
                      config.trigger_betterstack,
                      "SENTINEL@union.build",
                      "SIGNER_BALANCE_LOW",
                      "Union",
                      config.isLocal,
                    )
                    if (inc.data.id) {
                      markSignerIncident(db, key, inc.data.id)
                    }
                  }
                } else {
                  const logEffect = Effect.annotateLogs(tags)(Effect.logInfo("SIGNER_BALANCE_OK"))
                  Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))

                  if (existing) {
                    const didResolve = yield* resolveIncident(
                      existing,
                      config.betterstack_api_key,
                      config.trigger_betterstack,
                      config.isLocal,
                      "Sentinel-Automatically resolved.",
                    )
                    if (didResolve) {
                      clearSignerIncident(db, key)
                    }
                  }
                }
              }
            }
          })
          Effect.runFork(worker)
        }
      }
    }
  }),
  Schedule.spaced("30 minutes"),
)

const fetchMissingPackets = (hasuraEndpoint: string, exceedingSla: string) =>
  Effect.gen(function*() {
    let allPackets: Array<Packet> = []
    let cursor: string | undefined
    let continueFetching = true

    while (continueFetching) {
      let response: any

      if (cursor) {
        const queryNext = gql`
            query MissingPacketsNext($sla: String!, $cursor: String!) {
              v2_packets(args: {
                p_exceeding_sla: $sla,
                p_sort_order: $cursor
              }) {
                source_chain { universal_chain_id }
                destination_chain { universal_chain_id }
                packet_send_timestamp
                packet_hash
                status
                sort_order
              }
            }
          `
        response = yield* Effect.tryPromise({
          try: () =>
            request(hasuraEndpoint, queryNext, {
              sla: exceedingSla,
              cursor,
            }),
          catch: err => {
            console.error("fetchMissingPackets (next) failed:", err)
            return []
          },
        })
      } else {
        const queryFirst = gql`
            query MissingPackets($sla: String!) {
              v2_packets(args: { p_exceeding_sla: $sla }) {
                source_chain { universal_chain_id }
                destination_chain { universal_chain_id }
                packet_send_timestamp
                packet_hash
                status
                sort_order
              }
            }
          `
        response = yield* Effect.tryPromise({
          try: () =>
            request(hasuraEndpoint, queryFirst, {
              sla: exceedingSla,
            }),
          catch: err => {
            console.error("fetchMissingPackets (first) failed:", err)
            return []
          },
        })
      }

      const page: Array<Packet> = response.v2_packets || []
      if (page.length === 0) {
        break
      }

      allPackets.push(...page)
      // biome-ignore lint/style/noNonNullAssertion: <explanation>
      const last = page.at(-1)!

      cursor = last.sort_order
    }

    return allPackets
  })

export const checkPackets = (
  hasuraEndpoint: string,
  betterstack_api_key: string,
  trigger_betterstack: boolean,
  isLocal: boolean,
) =>
  Effect.gen(function*() {
    for (const sla of ["mainnet", "testnet"] as const) {
      if (sla == "testnet") {
        yield* Effect.log("Skipping testnet")
        continue
      }
      const transfer_error = sla === "mainnet" ? "MAINNET_TRANSFER_ERROR" : "TESTNET_TRANSFER_ERROR"
      const missingPacketsMainnet = yield* fetchMissingPackets(hasuraEndpoint, sla)
      if (!missingPacketsMainnet || missingPacketsMainnet.length === 0) {
        yield* Effect.log(`No missing packets found for ${sla}`)
        continue
      }
      yield* Effect.log(`Fetched ${missingPacketsMainnet.length} missingPackets from Hasura`)

      for (const missingPacket of missingPacketsMainnet) {
        const whole_description = {
          issueType: "TRANSFER_FAILED",
          currentStatus: missingPacket.status,
          sourceChain: missingPacket.source_chain.universal_chain_id,
          destinationChain: missingPacket.destination_chain.universal_chain_id,
          packetSendTimestamp: missingPacket.packet_send_timestamp,
          packetHash: missingPacket.packet_hash,
          explorerUrl: `https://btc.union.build/explorer/transfers/${missingPacket.packet_hash}`,
        }
        const logEffect = Effect.annotateLogs(whole_description)(Effect.logError(transfer_error))

        if (!hasErrorOpen(db, sla, missingPacket.packet_hash)) {
          const val = yield* triggerIncident(
            `${transfer_error}: https://btc.union.build/explorer/transfers/${missingPacket.packet_hash}`,
            JSON.stringify(whole_description),
            betterstack_api_key,
            trigger_betterstack,
            "SENTINEL@union.build",
            transfer_error,
            "Union",
            isLocal,
          )
          markTransferError(db, sla, missingPacket.packet_hash, val.data.id)
        }
        Effect.runFork(logEffect.pipe(Effect.provide(Logger.json)))
      }
      const openErrors = getOpenErrors(db, sla)

      const missingSet = new Set(missingPacketsMainnet.map(p => p.packet_hash))

      for (const { packet_hash, incident_id } of openErrors) {
        if (!missingSet.has(packet_hash)) {
          yield* Effect.log(`Auto-resolving incident for packet ${packet_hash}`)
          const didResolve = yield* resolveIncident(
            incident_id,
            betterstack_api_key,
            trigger_betterstack,
            isLocal,
            "Sentinel-Automatically resolved.",
          )
          if (didResolve) {
            clearTransferError(db, sla, packet_hash)
          }
        }
      }
    }
  }).pipe(Effect.withLogSpan("checkPackets"))

const runIbcChecksForever = Effect.gen(function*(_) {
  const { config } = yield* Config

  const schedule = Schedule.spaced(`${config.cycleIntervalMs / 1000 / 60} minutes`)

  const effectToRepeat = Effect.gen(function*(_) {
    yield* Effect.log("\n========== Starting IBC cross-chain checks ==========")

    yield* checkPackets(
      config.hasuraEndpoint,
      config.betterstack_api_key,
      config.trigger_betterstack,
      config.isLocal,
    )
  })

  return yield* Effect.repeat(effectToRepeat, schedule)
})

const mainEffect = Effect.gen(function*(_) {
  const argv = yield* Effect.sync(() =>
    yargs(hideBin(process.argv))
      .option("config", {
        alias: "c",
        type: "string",
        demandOption: true,
        describe: "Path to the configuration file",
      })
      .help()
      .alias("help", "h")
      .parseSync()
  )

  console.info("Loading config from", argv.config)
  const config = yield* loadConfig(argv.config)

  try {
    db = new Database(config.dbPath)
  } catch (err) {
    console.error("Error opening database:", err)
    throw err
  }

  db.prepare(`
    CREATE TABLE IF NOT EXISTS funded_txs (
      transaction_hash TEXT PRIMARY KEY
    )
  `).run()

  db.prepare(`
    CREATE TABLE IF NOT EXISTS transfer_errors (
      sla           TEXT    NOT NULL,
      packet_hash   TEXT PRIMARY KEY,
      incident_id   TEXT NOT NULL,
      inserted_at   INTEGER
    )
  `).run()

  db.prepare(`
    CREATE TABLE IF NOT EXISTS signer_incidents (
      key          TEXT PRIMARY KEY,    -- "url:port:plugin:wallet"
      incident_id  TEXT NOT NULL,
      inserted_at  INTEGER
    )
  `).run()

  db.prepare(`
    CREATE TABLE IF NOT EXISTS ssl_incidents (
      url          TEXT    PRIMARY KEY,
      incident_id  TEXT    NOT NULL,
      inserted_at  INTEGER
    )
  `).run()

  db.prepare(`
    CREATE TABLE IF NOT EXISTS supply_incidents (
      key            TEXT PRIMARY KEY,  -- e.g. "$srcChain:dstChain:token.denom"
      incident_id    TEXT NOT NULL,
      inserted_at    INTEGER
    )
  `).run()

  db.prepare(`
    CREATE TABLE IF NOT EXISTS aggregate_incidents (
      key            TEXT PRIMARY KEY,  -- e.g. "chainId:tokenAddr"
      incident_id    TEXT NOT NULL,
      inserted_at    INTEGER
    )
  `).run()

  yield* Effect.log("Database opened at", config.dbPath, "hasuraEndpoint:", config.hasuraEndpoint)

  yield* Effect.all(
    [
      // runIbcChecksForever,
      escrowSupplyControlLoop,
      // fundBabylonAccounts,
      // checkBalances,
      // checkSSLCertificates,
    ],
    {
      concurrency: "unbounded",
    },
  ).pipe(Effect.provideService(Config, { config }))
})

pipe(
  mainEffect,
  Effect.catchAllCause(cause =>
    Effect.sync(() => {
      console.error("💥 mainEffect failed:\n", Cause.pretty(cause))
    })
  ),
  Effect.runPromise,
).catch(err => {
  console.error("🔥 runPromise threw:", err.stack || err)
})
