import { HttpApi, HttpApiBuilder, OpenApi } from "@effect/platform"
import { Effect, Layer } from "effect"
import { HealthApi, ChainApi, AnalyticsApi, ProxyApi } from "./Api.js"
import { Database } from "./Db.js"
import { Sync } from "./Sync.js"
import { CHAINS, type ChainConfig } from "./config.js"

// Chain registry for O(1) lookup
const chainRegistry = new Map<string, ChainConfig>(CHAINS.map(c => [c.id, c]))

function getChain(chainId: string): ChainConfig | undefined {
  return chainRegistry.get(chainId)
}

// Compose the full API
export const IndexerApi = HttpApi.make("indexer")
  .add(HealthApi)
  .add(ChainApi)
  .add(AnalyticsApi)
  .add(ProxyApi)
  .annotate(OpenApi.Title, "Cosmos Indexer API")
  .annotate(OpenApi.Description, "Real-time blockchain indexer for Cosmos chains with full block and transaction data")
  .annotate(OpenApi.Version, "2.0.0")

// Health handlers
export const HttpHealthLive = HttpApiBuilder.group(IndexerApi, "health", (handlers) =>
  handlers
    .handle("health", () =>
      Effect.gen(function* () {
        const sync = yield* Sync
        const db = yield* Database
        const state = yield* sync.getState()
        const dbSize = yield* db.getDbSizeBytes()
        const chains = Object.entries(state.chains).map(([id, s]) => ({
          chain_id: id,
          name: getChain(id)?.name || id,
          ...s,
        }))
        const allSynced = chains.every((c) => c.status === "synced")
        const anyError = chains.some((c) => c.status === "error")

        return {
          status: anyError ? "degraded" : allSynced ? "healthy" : "syncing",
          started_at: state.startedAt,
          uptime_seconds: Math.floor((Date.now() - new Date(state.startedAt).getTime()) / 1000),
          db_size_bytes: dbSize,
          chains,
        } as const
      })
    )
    .handle("chains", () => Effect.succeed(CHAINS.map((c) => ({ id: c.id, name: c.name }))))
)

// Chain data handlers
export const HttpChainLive = HttpApiBuilder.group(IndexerApi, "chain", (handlers) =>
  handlers
    .handle("blocks", ({ path, urlParams }) =>
      Effect.gen(function* () {
        const db = yield* Database
        const blocks = yield* db.getBlocks(path.chainId, urlParams.limit, urlParams.before)
        return { blocks }
      })
    )
    .handle("blockByHeight", ({ path }) =>
      Effect.gen(function* () {
        const db = yield* Database
        const block = yield* db.getBlockByHeight(path.chainId, path.height)
        return { block }
      })
    )
    .handle("transactions", ({ path, urlParams }) =>
      Effect.gen(function* () {
        const db = yield* Database
        const txs = yield* db.getTransactions(path.chainId, urlParams.limit, urlParams.before)
        return { txs }
      })
    )
    .handle("transactionByHash", ({ path }) =>
      Effect.gen(function* () {
        const db = yield* Database
        const tx = yield* db.getTransactionByHash(path.chainId, path.hash)
        return { tx }
      })
    )
    .handle("transactionsByHeight", ({ path }) =>
      Effect.gen(function* () {
        const db = yield* Database
        const txs = yield* db.getTransactionsByHeight(path.chainId, path.height)
        return { txs }
      })
    )
    .handle("transactionsByAddress", ({ path, urlParams }) =>
      Effect.gen(function* () {
        const db = yield* Database
        const txs = yield* db.getTransactionsByAddress(path.chainId, path.address, urlParams.limit)
        return { txs }
      })
    )
    .handle("status", ({ path }) =>
      Effect.gen(function* () {
        const sync = yield* Sync
        const state = yield* sync.getChainState(path.chainId)
        if (!state) {
          return yield* Effect.fail(new Error("Chain not found"))
        }
        return {
          chain_id: path.chainId,
          name: getChain(path.chainId)?.name || path.chainId,
          ...state,
        }
      })
    )
)

// Analytics handlers
export const HttpAnalyticsLive = HttpApiBuilder.group(IndexerApi, "analytics", (handlers) =>
  handlers
    .handle("chainStats", ({ path }) =>
      Effect.gen(function* () {
        const db = yield* Database
        const stats = yield* db.getLatestChainStats(path.chainId)
        return { stats }
      })
    )
    .handle("chainStatsHistory", ({ path, urlParams }) =>
      Effect.gen(function* () {
        const db = yield* Database
        const stats = yield* db.getChainStatsHistory(path.chainId, urlParams.limit)
        return { stats }
      })
    )
    .handle("msgTypes", ({ path }) =>
      Effect.gen(function* () {
        const db = yield* Database
        const stats = yield* db.getMsgTypeStats(path.chainId)
        return { stats }
      })
    )
    .handle("daily", ({ path, urlParams }) =>
      Effect.gen(function* () {
        const db = yield* Database
        const stats = yield* db.getDailyStats(path.chainId, urlParams.days)
        return { stats }
      })
    )
    .handle("hourly", ({ path, urlParams }) =>
      Effect.gen(function* () {
        const db = yield* Database
        const stats = yield* db.getHourlyStats(path.chainId, urlParams.hours)
        return { stats }
      })
    )
)

// Helper to format block in Cosmos REST format
function formatBlockResponse(block: {
  hash: string
  header: unknown
  signatures: unknown
  tx_hashes: string[]
}) {
  return {
    block_id: { hash: block.hash },
    block: {
      header: block.header,
      data: { txs: block.tx_hashes },
      last_commit: { signatures: block.signatures },
    },
  }
}

// Helper to format tx in Cosmos REST format
function formatTxResponse(tx: {
  height: number
  hash: string
  code: number
  codespace: string
  gas_wanted: string
  gas_used: string
  raw_log: string
  events: unknown
  messages: unknown
  memo: string
  fee: unknown
  timestamp: string
}) {
  return {
    tx_response: {
      height: String(tx.height),
      txhash: tx.hash,
      code: tx.code,
      codespace: tx.codespace,
      gas_wanted: tx.gas_wanted,
      gas_used: tx.gas_used,
      raw_log: tx.raw_log,
      events: tx.events,
      tx: {
        body: { messages: tx.messages, memo: tx.memo },
        auth_info: { fee: tx.fee },
      },
      timestamp: tx.timestamp,
    },
  }
}

// Precompiled regex patterns for smart routing
const TX_BY_HASH_PATTERN = /^cosmos\/tx\/v1beta1\/txs\/([A-Fa-f0-9]+)$/
const TXS_BY_HEIGHT_PATTERN = /^cosmos\/tx\/v1beta1\/txs\?.*events=tx\.height=(\d+)/

// Shared proxy fetch helper
const proxyFetch = (baseUrl: string, path: string, label: string) =>
  Effect.gen(function* () {
    // Basic path validation - no directory traversal
    if (path.includes("..")) {
      return yield* Effect.fail(new Error("Invalid path"))
    }

    const url = `${baseUrl}/${path}`
    yield* Effect.log(`[${label}] -> ${url}`)

    const response = yield* Effect.tryPromise({
      try: () => fetch(url, { headers: { "Accept": "application/json" } }),
      catch: (e) => new Error(`Fetch failed: ${e instanceof Error ? e.message : e}`),
    })

    if (!response.ok) {
      return yield* Effect.fail(new Error(`Upstream ${response.status}`))
    }

    return yield* Effect.tryPromise({
      try: () => response.json(),
      catch: (e) => new Error(`JSON parse failed: ${e instanceof Error ? e.message : e}`),
    })
  })

// Proxy handlers - forward to chain REST, with smart routing for indexed data
export const HttpProxyLive = HttpApiBuilder.group(IndexerApi, "proxy", (handlers) =>
  handlers
    .handle("proxyRest", ({ path }) =>
      Effect.gen(function* () {
        const { chainId } = path
        const restPath = path["*"]
        const db = yield* Database
        const chain = getChain(chainId)

        if (!chain) {
          return yield* Effect.fail(new Error(`Unknown chain: ${chainId}`))
        }

        // Smart routing: serve from index if available

        // Pattern: /cosmos/tx/v1beta1/txs/{hash}
        const txByHashMatch = restPath.match(TX_BY_HASH_PATTERN)
        if (txByHashMatch) {
          const tx = yield* db.getTransactionByHash(chainId, txByHashMatch[1].toUpperCase())
          if (tx) return formatTxResponse(tx)
        }

        // Pattern: /cosmos/tx/v1beta1/txs?events=tx.height={height}
        const txsByHeightMatch = restPath.match(TXS_BY_HEIGHT_PATTERN)
        if (txsByHeightMatch) {
          const txs = yield* db.getTransactionsByHeight(chainId, parseInt(txsByHeightMatch[1], 10))
          if (txs.length > 0) {
            return {
              txs: txs.map((tx) => ({ body: { messages: tx.messages, memo: tx.memo }, auth_info: { fee: tx.fee } })),
              tx_responses: txs.map(formatTxResponse).map((r) => r.tx_response),
              pagination: { total: String(txs.length) },
            }
          }
        }

        // Pattern: /cosmos/staking/v1beta1/pool
        if (restPath === "cosmos/staking/v1beta1/pool") {
          const stats = yield* db.getLatestChainStats(chainId)
          if (stats) {
            return { pool: { bonded_tokens: stats.bonded_tokens, not_bonded_tokens: stats.not_bonded_tokens } }
          }
        }

        // Pattern: /cosmos/base/tendermint/v1beta1/syncing
        if (restPath === "cosmos/base/tendermint/v1beta1/syncing") {
          const sync = yield* Sync
          const state = yield* sync.getChainState(chainId)
          if (state) return { syncing: state.status !== "synced" }
        }

        // Fallback: proxy to chain
        return yield* proxyFetch(chain.rest[0], restPath, `proxy:${chainId}`)
      })
    )
    .handle("proxyRpc", ({ path }) =>
      Effect.gen(function* () {
        const { chainId } = path
        const rpcPath = path["*"]
        const chain = getChain(chainId)

        if (!chain) {
          return yield* Effect.fail(new Error(`Unknown chain: ${chainId}`))
        }

        return yield* proxyFetch(chain.rpc[0], rpcPath, `rpc:${chainId}`)
      })
    )
)

// Combined API layer
export const HttpApiLive = Layer.provide(HttpApiBuilder.api(IndexerApi), [
  HttpHealthLive,
  HttpChainLive,
  HttpAnalyticsLive,
  HttpProxyLive,
])
