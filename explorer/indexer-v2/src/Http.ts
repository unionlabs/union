import { HttpApi, HttpApiBuilder, OpenApi } from "@effect/platform"
import { Effect, Layer } from "effect"
import {
  AnalyticsApi,
  ChainApi,
  ChainNotFoundError,
  DatabaseError,
  HealthApi,
  NotFoundError,
  ProxyApi,
  UpstreamError,
} from "./Api.js"
import { type ChainConfig, CHAINS } from "./config.js"
import { Database } from "./Db.js"
import { DatabaseError as DbError } from "./errors.js"
import { Sync } from "./Sync.js"

// Chain registry for O(1) lookup
const chainRegistry = new Map<string, ChainConfig>(CHAINS.map((c) => [c.id, c]))

function getChain(chainId: string): ChainConfig | undefined {
  return chainRegistry.get(chainId)
}

// Validate chain exists - returns typed error
const validateChain = (chainId: string) =>
  Effect.gen(function*() {
    const chain = getChain(chainId)
    if (!chain) {
      return yield* Effect.fail(new ChainNotFoundError({ message: `Unknown chain: ${chainId}` }))
    }
    return chain
  })

// Map database errors to API errors
const mapDbError = (e: DbError) => new DatabaseError({ message: e.message })

// Compose the full API
export const IndexerApi = HttpApi.make("indexer")
  .add(HealthApi)
  .add(ChainApi)
  .add(AnalyticsApi)
  .add(ProxyApi)
  .annotate(OpenApi.Title, "Cosmos Indexer API")
  .annotate(
    OpenApi.Description,
    "Real-time blockchain indexer for Cosmos chains with full block and transaction data",
  )
  .annotate(OpenApi.Version, "2.0.0")

// Health handlers
export const HttpHealthLive = HttpApiBuilder.group(IndexerApi, "health", (handlers) =>
  handlers
    .handle("health", () =>
      Effect.gen(function*() {
        const sync = yield* Sync
        const db = yield* Database
        const state = yield* sync.getState()
        const dbSize = yield* db.getDbSizeBytes().pipe(Effect.mapError(mapDbError))
        const chains = Object.entries(state.chains).map(([id, s]) => ({
          chain_id: id,
          name: getChain(id)?.name ?? id,
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
      }))
    .handle("chains", () => Effect.succeed(CHAINS.map((c) => ({ id: c.id, name: c.name })))))

// Chain data handlers
export const HttpChainLive = HttpApiBuilder.group(IndexerApi, "chain", (handlers) =>
  handlers
    .handle("blocks", ({ path, urlParams }) =>
      Effect.gen(function*() {
        yield* validateChain(path.chainId)
        const db = yield* Database
        const blocks = yield* db
          .getBlocks(path.chainId, urlParams.limit, urlParams.before)
          .pipe(Effect.mapError(mapDbError))
        return { blocks }
      }))
    .handle("blockByHeight", ({ path }) =>
      Effect.gen(function*() {
        yield* validateChain(path.chainId)
        const db = yield* Database
        const block = yield* db
          .getBlockByHeight(path.chainId, path.height)
          .pipe(Effect.mapError(mapDbError))
        return { block }
      }))
    .handle("transactions", ({ path, urlParams }) =>
      Effect.gen(function*() {
        yield* validateChain(path.chainId)
        const db = yield* Database
        const txs = yield* db
          .getTransactions(path.chainId, urlParams.limit, urlParams.before)
          .pipe(Effect.mapError(mapDbError))
        return { txs }
      }))
    .handle("transactionByHash", ({ path }) =>
      Effect.gen(function*() {
        yield* validateChain(path.chainId)
        const db = yield* Database
        const tx = yield* db
          .getTransactionByHash(path.chainId, path.hash)
          .pipe(Effect.mapError(mapDbError))
        return { tx }
      }))
    .handle("transactionsByHeight", ({ path }) =>
      Effect.gen(function*() {
        yield* validateChain(path.chainId)
        const db = yield* Database
        const txs = yield* db
          .getTransactionsByHeight(path.chainId, path.height)
          .pipe(Effect.mapError(mapDbError))
        return { txs }
      }))
    .handle("transactionsByAddress", ({ path, urlParams }) =>
      Effect.gen(function*() {
        yield* validateChain(path.chainId)
        const db = yield* Database
        const txs = yield* db
          .getTransactionsByAddress(path.chainId, path.address, urlParams.limit)
          .pipe(Effect.mapError(mapDbError))
        return { txs }
      }))
    .handle("status", ({ path }) =>
      Effect.gen(function*() {
        const chain = yield* validateChain(path.chainId)
        const sync = yield* Sync
        const state = yield* sync.getChainState(path.chainId)
        if (!state) {
          return yield* Effect.fail(
            new ChainNotFoundError({ message: `Chain not found: ${path.chainId}` }),
          )
        }
        return {
          chain_id: path.chainId,
          name: chain.name,
          ...state,
        }
      })))

// Analytics handlers
export const HttpAnalyticsLive = HttpApiBuilder.group(
  IndexerApi,
  "analytics",
  (handlers) =>
    handlers
      .handle("chainStats", ({ path }) =>
        Effect.gen(function*() {
          yield* validateChain(path.chainId)
          const db = yield* Database
          const stats = yield* db
            .getLatestChainStats(path.chainId)
            .pipe(Effect.mapError(mapDbError))
          return { stats }
        }))
      .handle("chainStatsHistory", ({ path, urlParams }) =>
        Effect.gen(function*() {
          yield* validateChain(path.chainId)
          const db = yield* Database
          const stats = yield* db
            .getChainStatsHistory(path.chainId, urlParams.limit)
            .pipe(Effect.mapError(mapDbError))
          return { stats }
        }))
      .handle("msgTypes", ({ path }) =>
        Effect.gen(function*() {
          yield* validateChain(path.chainId)
          const db = yield* Database
          const stats = yield* db.getMsgTypeStats(path.chainId).pipe(Effect.mapError(mapDbError))
          return { stats }
        }))
      .handle("daily", ({ path, urlParams }) =>
        Effect.gen(function*() {
          yield* validateChain(path.chainId)
          const db = yield* Database
          const stats = yield* db
            .getDailyStats(path.chainId, urlParams.days)
            .pipe(Effect.mapError(mapDbError))
          return { stats }
        }))
      .handle("hourly", ({ path, urlParams }) =>
        Effect.gen(function*() {
          yield* validateChain(path.chainId)
          const db = yield* Database
          const stats = yield* db
            .getHourlyStats(path.chainId, urlParams.hours)
            .pipe(Effect.mapError(mapDbError))
          return { stats }
        })),
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
const BLOCK_BY_HEIGHT_PATTERN = /^cosmos\/base\/tendermint\/v1beta1\/blocks\/(\d+)$/
const LATEST_BLOCK_PATTERN = /^cosmos\/base\/tendermint\/v1beta1\/blocks\/latest$/

// Cache TTL: 7 days in seconds
const CACHE_TTL_IMMUTABLE = 7 * 24 * 60 * 60

// Determine if a path should be cached and for how long
function getCacheTTL(path: string): number | null {
  // Don't cache "latest" or real-time endpoints
  if (path.includes("latest") || path.includes("syncing")) {
    return null
  }
  // Blocks by height - immutable
  if (BLOCK_BY_HEIGHT_PATTERN.test(path)) {
    return CACHE_TTL_IMMUTABLE
  }
  // Txs by hash - immutable
  if (TX_BY_HASH_PATTERN.test(path)) {
    return CACHE_TTL_IMMUTABLE
  }
  // Txs by height - immutable
  if (TXS_BY_HEIGHT_PATTERN.test(path)) {
    return CACHE_TTL_IMMUTABLE
  }
  // Default: don't cache unknown endpoints
  return null
}

// Shared proxy fetch helper - races all endpoints for fastest response
const proxyFetch = (endpoints: string[], path: string, label: string) =>
  Effect.gen(function*() {
    // Basic path validation - no directory traversal
    if (path.includes("..")) {
      return yield* Effect.fail(new NotFoundError({ message: "Invalid path" }))
    }

    if (endpoints.length === 0) {
      return yield* Effect.fail(new UpstreamError({ message: "No endpoints configured" }))
    }

    yield* Effect.logDebug(`[${label}] Racing ${endpoints.length} endpoints for: ${path}`)

    const result = yield* Effect.tryPromise({
      try: async () => {
        const controller = new AbortController()
        const timeout = setTimeout(() => controller.abort(), 15000)

        try {
          // Race all endpoints - first successful response wins
          return await Promise.any(
            endpoints.map(async (baseUrl) => {
              const url = `${baseUrl}/${path}`
              const response = await fetch(url, {
                headers: { Accept: "application/json" },
                signal: controller.signal,
              })

              if (response.status === 429) {
                throw new Error("Rate limited")
              }

              if (response.status === 404) {
                throw new Error("NOT_FOUND")
              }

              if (!response.ok) {
                const body = await response.text().catch(() => "")
                const lowerBody = body.toLowerCase()
                if (
                  lowerBody.includes("not found")
                  || lowerBody.includes("does not exist")
                  || lowerBody.includes("pruned")
                  || lowerBody.includes("is not available")
                  || lowerBody.includes("height must be less than or equal")
                  || response.status === 400
                ) {
                  throw new Error("NOT_FOUND")
                }
                throw new Error(`HTTP ${response.status}: ${body.slice(0, 100)}`)
              }

              return response.json()
            }),
          )
        } finally {
          clearTimeout(timeout)
        }
      },
      catch: (e) => {
        // AggregateError from Promise.any means all failed
        if (e instanceof AggregateError) {
          const errors = e.errors.filter((err): err is Error => err instanceof Error)
          const notFoundCount = errors.filter((err) => err.message === "NOT_FOUND").length

          // Only return 404 if ALL endpoints returned "not found"
          // If any returned a transient error (429, timeout, etc.), return 502 so client can retry
          if (notFoundCount === errors.length && errors.length > 0) {
            return new NotFoundError({ message: `Resource not found: ${path}` })
          }

          // Some endpoints had transient errors - return 502
          const transientErrors = errors.filter((err) => err.message !== "NOT_FOUND")
          const lastErr = transientErrors[0] ?? errors[0]
          return new UpstreamError({
            message: lastErr?.message ?? "All endpoints failed",
          })
        }
        if (e instanceof Error && e.message === "NOT_FOUND") {
          return new NotFoundError({ message: `Resource not found: ${path}` })
        }
        return new UpstreamError({
          message: e instanceof Error ? e.message : String(e),
        })
      },
    })

    // Check if result is an error type (from catch block)
    if (result instanceof NotFoundError || result instanceof UpstreamError) {
      return yield* Effect.fail(result)
    }

    return result
  })

// Proxy handlers - forward to chain REST, with smart routing for indexed data
export const HttpProxyLive = HttpApiBuilder.group(IndexerApi, "proxy", (handlers) =>
  handlers
    .handle("proxyRest", ({ path }) =>
      Effect.gen(function*() {
        const { chainId } = path
        const restPath = path["*"]
        const db = yield* Database
        const chain = yield* validateChain(chainId)

        // Smart routing: serve from index if available

        // Pattern: /cosmos/tx/v1beta1/txs/{hash}
        const txByHashMatch = restPath.match(TX_BY_HASH_PATTERN)
        if (txByHashMatch) {
          const hashPart = txByHashMatch[1]
          if (hashPart) {
            const tx = yield* db
              .getTransactionByHash(chainId, hashPart.toUpperCase())
              .pipe(Effect.catchAll(() => Effect.succeed(null)))
            if (tx) {
              return formatTxResponse(tx)
            }
          }
        }

        // Pattern: /cosmos/tx/v1beta1/txs?events=tx.height={height}
        const txsByHeightMatch = restPath.match(TXS_BY_HEIGHT_PATTERN)
        if (txsByHeightMatch) {
          const heightStr = txsByHeightMatch[1]
          if (heightStr) {
            const txs = yield* db
              .getTransactionsByHeight(chainId, parseInt(heightStr, 10))
              .pipe(Effect.catchAll(() => Effect.succeed([])))
            if (txs.length > 0) {
              return {
                txs: txs.map((tx) => ({
                  body: { messages: tx.messages, memo: tx.memo },
                  auth_info: { fee: tx.fee },
                })),
                tx_responses: txs.map(formatTxResponse).map((r) => r.tx_response),
                pagination: { total: String(txs.length) },
              }
            }
          }
        }

        // Pattern: /cosmos/staking/v1beta1/pool
        if (restPath === "cosmos/staking/v1beta1/pool") {
          const stats = yield* db
            .getLatestChainStats(chainId)
            .pipe(Effect.catchAll(() => Effect.succeed(null)))
          if (stats) {
            return {
              pool: {
                bonded_tokens: stats.bonded_tokens,
                not_bonded_tokens: stats.not_bonded_tokens,
              },
            }
          }
        }

        // Pattern: /cosmos/base/tendermint/v1beta1/syncing
        if (restPath === "cosmos/base/tendermint/v1beta1/syncing") {
          const sync = yield* Sync
          const state = yield* sync.getChainState(chainId)
          if (state) {
            return { syncing: state.status !== "synced" }
          }
        }

        // Pattern: /cosmos/base/tendermint/v1beta1/blocks/{height}
        const blockByHeightMatch = restPath.match(BLOCK_BY_HEIGHT_PATTERN)
        if (blockByHeightMatch) {
          const heightStr = blockByHeightMatch[1]
          if (heightStr) {
            const height = parseInt(heightStr, 10)
            const block = yield* db
              .getBlockByHeight(chainId, height)
              .pipe(Effect.catchAll(() => Effect.succeed(null)))
            if (block) {
              return formatBlockResponse(block)
            }
          }
          // Block not in index - will try upstream below
        }

        // Pattern: /cosmos/base/tendermint/v1beta1/blocks/latest
        if (LATEST_BLOCK_PATTERN.test(restPath)) {
          // Get latest from index
          const blocks = yield* db
            .getBlocks(chainId, 1, undefined)
            .pipe(Effect.catchAll(() => Effect.succeed([])))
          if (blocks.length > 0 && blocks[0]) {
            return formatBlockResponse(blocks[0])
          }
        }

        // Fallback: proxy to chain with caching
        const cacheKey = `${chainId}:${restPath}`
        const ttl = getCacheTTL(restPath)

        // Check cache first (only for cacheable paths)
        if (ttl) {
          const cached = yield* db.getCached(cacheKey).pipe(
            Effect.catchAll(() => Effect.succeed(null)),
          )
          if (cached) {
            yield* Effect.logDebug(`[proxy:${chainId}] Cache hit: ${restPath}`)
            return JSON.parse(cached)
          }
        }

        // Fetch from upstream
        const result = yield* proxyFetch(chain.rest, restPath, `proxy:${chainId}`)

        // Cache the result (only for cacheable paths)
        if (ttl) {
          yield* db
            .setCache(cacheKey, JSON.stringify(result), ttl)
            .pipe(Effect.catchAll(() => Effect.void))
        }

        return result
      }))
    .handle("proxyRpc", ({ path }) =>
      Effect.gen(function*() {
        const { chainId } = path
        const rpcPath = path["*"]
        const db = yield* Database
        const chain = yield* validateChain(chainId)

        const cacheKey = `rpc:${chainId}:${rpcPath}`
        const ttl = getCacheTTL(rpcPath)

        // Check cache first
        if (ttl) {
          const cached = yield* db.getCached(cacheKey).pipe(
            Effect.catchAll(() => Effect.succeed(null)),
          )
          if (cached) {
            return JSON.parse(cached)
          }
        }

        const result = yield* proxyFetch(chain.rpc, rpcPath, `rpc:${chainId}`)

        // Cache the result
        if (ttl) {
          yield* db
            .setCache(cacheKey, JSON.stringify(result), ttl)
            .pipe(Effect.catchAll(() => Effect.void))
        }

        return result
      })))

// Combined API layer
export const HttpApiLive = Layer.provide(HttpApiBuilder.api(IndexerApi), [
  HttpHealthLive,
  HttpChainLive,
  HttpAnalyticsLive,
  HttpProxyLive,
])
