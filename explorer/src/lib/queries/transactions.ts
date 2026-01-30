import { CosmosClient } from "$lib/services/cosmos-client"
import type { PaginationResponse, TxResponse } from "$lib/types/cosmos"
import { Effect } from "effect"

export const fetchTransaction = (hash: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getTx(hash)
  })

export const fetchTransactionsByHeight = (height: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getTxsByHeight(height)
  })

export const searchTransactions = (query: string, page = 1, limit = 20) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.searchTxs(query, page, limit)
  })

// Fetch recent transactions via RPC
export const fetchRecentTransactionsGlobal = (limit = 50) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient

    // Get recent blocks (scan more to find enough with txs)
    const latest = yield* client.getLatestBlock()
    const latestHeight = parseInt(latest.block.header.height)

    // Scan recent blocks to find ones with transactions
    const scanCount = Math.min(200, latestHeight) // Scan up to 200 blocks
    const blockRange = yield* client.getBlockRange(latestHeight - scanCount + 1, latestHeight)

    // Filter to blocks that have transactions
    const blocksWithTxs = blockRange.filter((b) => b.txCount > 0).slice(0, 20) // Limit blocks to query

    if (blocksWithTxs.length === 0) {
      return { txs: [], tx_responses: [], pagination: { total: "0", next_key: null } }
    }

    // Fetch transactions from each block (using exact height = fast query)
    const txResults = yield* Effect.all(
      blocksWithTxs.map((block) => client.getTxsByHeight(block.height)),
      { concurrency: 5 },
    )

    // Flatten and limit
    const allTxs: TxResponse[] = txResults.flatMap((r) => r.tx_responses).slice(0, limit)

    return {
      txs: [],
      tx_responses: allTxs,
      pagination: { total: String(allTxs.length), next_key: null } as PaginationResponse,
    }
  })

// Fetch more transactions starting from a specific height
export const fetchTransactionsPage = (page: number, limit = 50) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient

    // For pagination, we scan older blocks
    const latest = yield* client.getLatestBlock()
    const latestHeight = parseInt(latest.block.header.height)

    // Skip blocks based on page (rough approximation)
    const blocksPerPage = 200
    const startHeight = Math.max(1, latestHeight - page * blocksPerPage)
    const endHeight = Math.max(1, startHeight - blocksPerPage + 1)

    if (startHeight <= 1) {
      return { txs: [], tx_responses: [], pagination: { total: "0", next_key: null } }
    }

    const blockRange = yield* client.getBlockRange(endHeight, startHeight)
    const blocksWithTxs = blockRange.filter((b) => b.txCount > 0).slice(0, 20)

    if (blocksWithTxs.length === 0) {
      return { txs: [], tx_responses: [], pagination: { total: "0", next_key: null } }
    }

    const txResults = yield* Effect.all(
      blocksWithTxs.map((block) => client.getTxsByHeight(block.height)),
      { concurrency: 5 },
    )

    const allTxs: TxResponse[] = txResults.flatMap((r) => r.tx_responses).slice(0, limit)

    return {
      txs: [],
      tx_responses: allTxs,
      pagination: { total: String(allTxs.length), next_key: null } as PaginationResponse,
    }
  })
