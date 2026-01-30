import { CosmosClient } from "$lib/services/cosmos-client"
import type { Block } from "$lib/types/cosmos"
import { Effect } from "effect"

export const fetchLatestBlock = () =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getLatestBlock()
  })

export const fetchBlockByHeight = (height: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getBlockByHeight(height)
  })

// Lightweight block info for list views
export interface BlockSummary {
  height: string
  time: string
  hash: string
  proposer: string
  txCount: number
}

// Fetch recent blocks using bulk RPC endpoint (much faster, fewer requests)
export const fetchRecentBlocksBulk = (count = 50) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    const latest = yield* client.getLatestBlock()
    const latestHeight = parseInt(latest.block.header.height)

    // RPC returns max 20 blocks per request, so chunk the requests
    const chunkSize = 20
    const chunks: Array<{ min: number; max: number }> = []

    for (let i = 0; i < count; i += chunkSize) {
      const max = latestHeight - i
      const min = Math.max(latestHeight - i - chunkSize + 1, 1)
      chunks.push({ min, max })
    }

    // Fetch all chunks in parallel
    const chunkResults = yield* Effect.all(
      chunks.map(({ min, max }) => client.getBlockRange(min, max)),
      { concurrency: 3 },
    )

    // Flatten and take only what we need
    const allBlocks = chunkResults.flat()
    return allBlocks.slice(0, count)
  })

// Fetch blocks starting from a specific height going backwards
export const fetchBlockRangeFromHeight = (startHeight: number, count = 50) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient

    // RPC returns max 20 blocks per request, so chunk the requests
    const chunkSize = 20
    const chunks: Array<{ min: number; max: number }> = []

    for (let i = 0; i < count; i += chunkSize) {
      const max = startHeight - i
      const min = Math.max(startHeight - i - chunkSize + 1, 1)
      if (max < 1) {
        break
      }
      chunks.push({ min, max })
    }

    if (chunks.length === 0) {
      return []
    }

    // Fetch all chunks in parallel
    const chunkResults = yield* Effect.all(
      chunks.map(({ min, max }) => client.getBlockRange(min, max)),
      { concurrency: 3 },
    )

    // Flatten and take only what we need
    const allBlocks = chunkResults.flat()
    return allBlocks.slice(0, count)
  })

// Fetch full blocks with commits/signatures (for validator uptime)
// Fetches in batches to avoid rate limits
export const fetchRecentBlocksFull = (count = 100) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    const latest = yield* client.getLatestBlock()
    const latestHeight = parseInt(latest.block.header.height)

    const allBlocks: Block[] = [latest]
    const batchSize = 10

    // Fetch remaining blocks in batches
    for (let i = 1; i < count; i += batchSize) {
      const heights = Array.from(
        { length: Math.min(batchSize, count - i) },
        (_, j) => latestHeight - i - j,
      )
      const blockEffects = heights.map((h) => client.getBlockByHeight(String(h)))
      const results = yield* Effect.all(blockEffects, { concurrency: 5 })
      allBlocks.push(...results)
    }

    return allBlocks
  })

// Legacy: Fetch recent blocks one by one (slower, for fallback)
export const fetchRecentBlocks = (count = 20) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    const latest = yield* client.getLatestBlock()
    const latestHeight = parseInt(latest.block.header.height)

    const blocks: Block[] = [latest]

    // Fetch previous blocks in parallel (limited to avoid rate limits)
    const heights = Array.from({ length: Math.min(count - 1, 19) }, (_, i) => latestHeight - i - 1)
    const blockEffects = heights.map((h) => client.getBlockByHeight(String(h)))

    const results = yield* Effect.allSuccesses(blockEffects)
    blocks.push(...results)

    return blocks
  })

export const fetchValidatorSet = (height?: string) =>
  Effect.gen(function*() {
    const client = yield* CosmosClient
    return yield* client.getValidatorSet(height)
  })
