import { HttpApiEndpoint, HttpApiGroup, OpenApi } from "@effect/platform"
import { Schema } from "effect"

// ============ Schemas ============

export const BlockSchema = Schema.Struct({
  chain_id: Schema.String,
  height: Schema.Number,
  hash: Schema.String,
  time: Schema.String,
  proposer: Schema.String,
  tx_count: Schema.Number,
  header: Schema.Unknown,
  signatures: Schema.Unknown,
  tx_hashes: Schema.Array(Schema.String),
})

export const TransactionSchema = Schema.Struct({
  chain_id: Schema.String,
  hash: Schema.String,
  height: Schema.Number,
  index: Schema.Number,
  code: Schema.Number,
  codespace: Schema.String,
  gas_used: Schema.String,
  gas_wanted: Schema.String,
  messages: Schema.Unknown,
  memo: Schema.String,
  fee: Schema.Unknown,
  events: Schema.Unknown,
  raw_log: Schema.String,
  tx_bytes: Schema.String,
  timestamp: Schema.String,
})

export const ChainSyncState = Schema.Struct({
  status: Schema.Literal("idle", "backfilling", "syncing", "synced", "error"),
  latestHeight: Schema.Number,
  indexedHeight: Schema.Number,
  blocksIndexed: Schema.Number,
  txsIndexed: Schema.Number,
  backfillProgress: Schema.Number,
  lastSync: Schema.NullOr(Schema.String),
  lastError: Schema.NullOr(Schema.String),
})

export const ChainInfo = Schema.Struct({
  chain_id: Schema.String,
  name: Schema.String,
}).pipe(Schema.extend(ChainSyncState))

export const HealthResponse = Schema.Struct({
  status: Schema.Literal("healthy", "syncing", "degraded"),
  started_at: Schema.String,
  uptime_seconds: Schema.Number,
  db_size_bytes: Schema.Number,
  chains: Schema.Array(ChainInfo),
})

export const ChainListItem = Schema.Struct({
  id: Schema.String,
  name: Schema.String,
})

export const BlocksResponse = Schema.Struct({
  blocks: Schema.Array(BlockSchema),
})

export const BlockResponse = Schema.Struct({
  block: Schema.NullOr(BlockSchema),
})

export const TransactionsResponse = Schema.Struct({
  txs: Schema.Array(TransactionSchema),
})

export const TransactionResponse = Schema.Struct({
  tx: Schema.NullOr(TransactionSchema),
})

export const MsgTypeStats = Schema.Struct({
  msg_type: Schema.NullOr(Schema.String),
  count: Schema.Number,
  success_count: Schema.Number,
  failure_count: Schema.Number,
})

export const MsgTypeStatsResponse = Schema.Struct({
  stats: Schema.Array(MsgTypeStats),
})

export const DailyStats = Schema.Struct({
  date: Schema.String,
  block_count: Schema.Number,
  tx_count: Schema.Number,
})

export const DailyStatsResponse = Schema.Struct({
  stats: Schema.Array(DailyStats),
})

export const HourlyStats = Schema.Struct({
  hour: Schema.String,
  count: Schema.Number,
})

export const HourlyStatsResponse = Schema.Struct({
  stats: Schema.Array(HourlyStats),
})

export const ChainStatusResponse = Schema.Struct({
  chain_id: Schema.String,
  name: Schema.String,
}).pipe(Schema.extend(ChainSyncState))

// ============ API Groups ============

export class HealthApi extends HttpApiGroup.make("health", { topLevel: true })
  .add(
    HttpApiEndpoint.get("health", "/health")
      .addSuccess(HealthResponse)
      .annotate(OpenApi.Summary, "Get indexer health and sync status")
  )
  .add(
    HttpApiEndpoint.get("chains", "/chains")
      .addSuccess(Schema.Array(ChainListItem))
      .annotate(OpenApi.Summary, "List all indexed chains")
  )
  .annotateContext(OpenApi.annotations({ title: "Health", description: "Health and status endpoints" })) {}

export class ChainApi extends HttpApiGroup.make("chain")
  .add(
    HttpApiEndpoint.get("blocks", "/:chainId/blocks")
      .setPath(Schema.Struct({ chainId: Schema.String }))
      .setUrlParams(Schema.Struct({
        limit: Schema.optionalWith(Schema.NumberFromString, { default: () => 50 }),
        before: Schema.optional(Schema.NumberFromString), // cursor: get blocks before this height
      }))
      .addSuccess(BlocksResponse)
      .annotate(OpenApi.Summary, "Get recent blocks for a chain")
  )
  .add(
    HttpApiEndpoint.get("blockByHeight", "/:chainId/block/:height")
      .setPath(Schema.Struct({ chainId: Schema.String, height: Schema.NumberFromString }))
      .addSuccess(BlockResponse)
      .annotate(OpenApi.Summary, "Get full block by height")
  )
  .add(
    HttpApiEndpoint.get("transactions", "/:chainId/txs")
      .setPath(Schema.Struct({ chainId: Schema.String }))
      .setUrlParams(Schema.Struct({
        limit: Schema.optionalWith(Schema.NumberFromString, { default: () => 50 }),
        before: Schema.optional(Schema.NumberFromString), // cursor: get txs before this height
      }))
      .addSuccess(TransactionsResponse)
      .annotate(OpenApi.Summary, "Get recent transactions for a chain")
  )
  .add(
    HttpApiEndpoint.get("transactionByHash", "/:chainId/tx/:hash")
      .setPath(Schema.Struct({ chainId: Schema.String, hash: Schema.String }))
      .addSuccess(TransactionResponse)
      .annotate(OpenApi.Summary, "Get full transaction by hash")
  )
  .add(
    HttpApiEndpoint.get("transactionsByHeight", "/:chainId/block/:height/txs")
      .setPath(Schema.Struct({ chainId: Schema.String, height: Schema.NumberFromString }))
      .addSuccess(TransactionsResponse)
      .annotate(OpenApi.Summary, "Get all transactions in a block")
  )
  .add(
    HttpApiEndpoint.get("transactionsByAddress", "/:chainId/address/:address/txs")
      .setPath(Schema.Struct({ chainId: Schema.String, address: Schema.String }))
      .setUrlParams(Schema.Struct({ limit: Schema.optionalWith(Schema.NumberFromString, { default: () => 50 }) }))
      .addSuccess(TransactionsResponse)
      .annotate(OpenApi.Summary, "Get transactions for an address")
  )
  .add(
    HttpApiEndpoint.get("status", "/:chainId/status")
      .setPath(Schema.Struct({ chainId: Schema.String }))
      .addSuccess(ChainStatusResponse)
      .annotate(OpenApi.Summary, "Get sync status for a chain")
  )
  .annotateContext(OpenApi.annotations({ title: "Chain Data", description: "Block and transaction data per chain" })) {}

export const ChainStatsSchema = Schema.Struct({
  chain_id: Schema.String,
  height: Schema.Number,
  timestamp: Schema.String,
  total_supply: Schema.String,
  bonded_tokens: Schema.String,
  not_bonded_tokens: Schema.String,
  inflation: Schema.String,
  community_pool: Schema.String,
})

export const ChainStatsResponse = Schema.Struct({
  stats: Schema.NullOr(ChainStatsSchema),
})

export const ChainStatsHistoryResponse = Schema.Struct({
  stats: Schema.Array(ChainStatsSchema),
})

export class AnalyticsApi extends HttpApiGroup.make("analytics")
  .add(
    HttpApiEndpoint.get("chainStats", "/:chainId/analytics/stats")
      .setPath(Schema.Struct({ chainId: Schema.String }))
      .addSuccess(ChainStatsResponse)
      .annotate(OpenApi.Summary, "Get latest chain stats (supply, staking)")
  )
  .add(
    HttpApiEndpoint.get("chainStatsHistory", "/:chainId/analytics/stats/history")
      .setPath(Schema.Struct({ chainId: Schema.String }))
      .setUrlParams(Schema.Struct({ limit: Schema.optionalWith(Schema.NumberFromString, { default: () => 100 }) }))
      .addSuccess(ChainStatsHistoryResponse)
      .annotate(OpenApi.Summary, "Get chain stats history")
  )
  .add(
    HttpApiEndpoint.get("msgTypes", "/:chainId/analytics/msg-types")
      .setPath(Schema.Struct({ chainId: Schema.String }))
      .addSuccess(MsgTypeStatsResponse)
      .annotate(OpenApi.Summary, "Get message type statistics")
  )
  .add(
    HttpApiEndpoint.get("daily", "/:chainId/analytics/daily")
      .setPath(Schema.Struct({ chainId: Schema.String }))
      .setUrlParams(Schema.Struct({ days: Schema.optionalWith(Schema.NumberFromString, { default: () => 7 }) }))
      .addSuccess(DailyStatsResponse)
      .annotate(OpenApi.Summary, "Get daily block/tx statistics")
  )
  .add(
    HttpApiEndpoint.get("hourly", "/:chainId/analytics/hourly")
      .setPath(Schema.Struct({ chainId: Schema.String }))
      .setUrlParams(Schema.Struct({ hours: Schema.optionalWith(Schema.NumberFromString, { default: () => 24 }) }))
      .addSuccess(HourlyStatsResponse)
      .annotate(OpenApi.Summary, "Get hourly transaction statistics")
  )
  .annotateContext(OpenApi.annotations({ title: "Analytics", description: "Chain analytics and statistics" })) {}

// Proxy API - forwards requests to chain RPC/REST
export class ProxyApi extends HttpApiGroup.make("proxy")
  .add(
    HttpApiEndpoint.get("proxyRest", "/:chainId/rest/*")
      .setPath(Schema.Struct({ chainId: Schema.String, "*": Schema.String }))
      .addSuccess(Schema.Unknown)
      .annotate(OpenApi.Summary, "Proxy request to chain REST API")
  )
  .add(
    HttpApiEndpoint.get("proxyRpc", "/:chainId/rpc/*")
      .setPath(Schema.Struct({ chainId: Schema.String, "*": Schema.String }))
      .addSuccess(Schema.Unknown)
      .annotate(OpenApi.Summary, "Proxy request to chain RPC API")
  )
  .annotateContext(OpenApi.annotations({ title: "Proxy", description: "Proxy requests to chain RPC/REST endpoints" })) {}
