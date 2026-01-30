import { Effect, Context, Layer, Ref, Duration } from "effect"
import { Database, type Block, type Transaction } from "./Db.js"
import { IndexerConfigService, CHAINS, type ChainConfig } from "./config.js"
import { createRpcClient, type RestBlock, type RestTxResponse } from "./Rpc.js"

// ============ Sync State ============

export interface ChainSyncState {
  status: "idle" | "backfilling" | "syncing" | "synced" | "error"
  latestHeight: number
  indexedHeight: number
  blocksIndexed: number
  txsIndexed: number
  backfillProgress: number
  lastSync: string | null
  lastError: string | null
}

export interface SyncState {
  startedAt: string
  chains: Record<string, ChainSyncState>
}

// ============ Service Interface ============

export interface SyncService {
  getState(): Effect.Effect<SyncState>
  getChainState(chainId: string): Effect.Effect<ChainSyncState | null>
  start(): Effect.Effect<void>
}

export class Sync extends Context.Tag("Sync")<Sync, SyncService>() {}

// ============ Data Converters ============

function restBlockToBlock(chainId: string, restBlock: RestBlock): Block {
  const { block, block_id } = restBlock
  return {
    chain_id: chainId,
    height: parseInt(block.header.height),
    hash: block_id.hash,
    time: block.header.time,
    proposer: block.header.proposer_address,
    tx_count: block.data.txs?.length ?? 0,
    header: block.header,
    signatures: block.last_commit?.signatures ?? [],
    tx_hashes: [], // Populated after fetching txs
  }
}

function restTxToTransaction(chainId: string, tx: RestTxResponse, index: number): Transaction {
  return {
    chain_id: chainId,
    hash: tx.txhash,
    height: parseInt(tx.height),
    index,
    code: tx.code,
    codespace: tx.codespace || "",
    gas_used: tx.gas_used,
    gas_wanted: tx.gas_wanted,
    messages: tx.tx?.body?.messages ?? [],
    memo: tx.tx?.body?.memo ?? "",
    fee: tx.tx?.auth_info?.fee ?? {},
    events: tx.events ?? [],
    raw_log: tx.raw_log || "",
    tx_bytes: "", // Could store if needed
    timestamp: tx.timestamp || "",
  }
}

// ============ Implementation ============

export const SyncLive = Layer.effect(
  Sync,
  Effect.gen(function* () {
    const db = yield* Database
    const config = yield* IndexerConfigService

    const startedAt = new Date().toISOString()

    // Initialize state
    const initial: Record<string, ChainSyncState> = {}
    for (const chain of CHAINS) {
      initial[chain.id] = {
        status: "idle",
        latestHeight: 0,
        indexedHeight: 0,
        blocksIndexed: 0,
        txsIndexed: 0,
        backfillProgress: 0,
        lastSync: null,
        lastError: null,
      }
    }
    const stateRef = yield* Ref.make(initial)

    const updateState = (chainId: string, update: Partial<ChainSyncState>) =>
      Ref.update(stateRef, (s) => ({ ...s, [chainId]: { ...s[chainId], ...update } }))

    // Fetch full block with all txs - logs errors but continues (resilient)
    const fetchFullBlockWithTxs = (rpc: ReturnType<typeof createRpcClient>, chainId: string, height: number) =>
      Effect.gen(function* () {
        const restBlock = yield* rpc.getFullBlock(height).pipe(
          Effect.catchAll((e) =>
            Effect.log(`[${chainId}] Block ${height} fetch failed: ${e}`).pipe(
              Effect.as(null)
            )
          )
        )

        if (!restBlock) return { block: null, txs: [] }

        const block = restBlockToBlock(chainId, restBlock)
        const txs: Transaction[] = []

        if (block.tx_count > 0) {
          const txResponse = yield* rpc.getTxsByHeight(height).pipe(
            Effect.catchAll((e) =>
              Effect.log(`[${chainId}] Txs for block ${height} fetch failed: ${e}`).pipe(
                Effect.as({ tx_responses: [] })
              )
            )
          )

          for (let i = 0; i < (txResponse.tx_responses?.length ?? 0); i++) {
            txs.push(restTxToTransaction(chainId, txResponse.tx_responses[i], i))
          }
          block.tx_hashes = txs.map((t) => t.hash)
        }

        return { block, txs }
      })

    // Shared batch fetch logic
    const fetchBlockBatch = (rpc: ReturnType<typeof createRpcClient>, chainId: string, heights: number[]) =>
      Effect.all(
        heights.map((h) => fetchFullBlockWithTxs(rpc, chainId, h)),
        { concurrency: 5 }
      ).pipe(
        Effect.map((results) => {
          const blocks: Block[] = []
          const txs: Transaction[] = []
          for (const r of results) {
            if (r.block) {
              blocks.push(r.block)
              txs.push(...r.txs)
            }
          }
          return { blocks, txs }
        })
      )

    // Backfill chain
    const backfillChain = (chain: ChainConfig) =>
      Effect.gen(function* () {
        yield* updateState(chain.id, { status: "backfilling" })
        const rpc = createRpcClient(chain)

        const status = yield* rpc.getStatus()
        const latestHeight = parseInt(status.result.sync_info.latest_block_height)
        const targetMin = Math.max(1, latestHeight - config.blocksToKeep + 1)
        const currentMax = yield* db.getLatestHeight(chain.id)

        yield* updateState(chain.id, { latestHeight })

        if (currentMax >= latestHeight - 10) {
          yield* updateState(chain.id, { status: "synced", backfillProgress: 100, indexedHeight: currentMax })
          yield* Effect.log(`[${chain.id}] Already synced`)
          return
        }

        const currentMin = yield* db.getMinHeight(chain.id)
        let height = currentMin > 0 ? currentMin - 1 : latestHeight

        yield* Effect.log(`[${chain.id}] Backfilling from ${height} -> ${targetMin}`)

        while (height > targetMin) {
          const batchEnd = Math.max(targetMin, height - config.backfillBatchSize + 1)
          const heights = Array.from({ length: height - batchEnd + 1 }, (_, i) => height - i)

          const { blocks, txs } = yield* fetchBlockBatch(rpc, chain.id, heights)
          yield* db.insertBlocks(blocks)
          yield* db.insertTransactions(txs)

          const [blockCount, maxHeight] = yield* Effect.all([
            db.getBlockCount(chain.id),
            db.getLatestHeight(chain.id),
          ])
          yield* updateState(chain.id, {
            backfillProgress: Math.min(100, Math.round((blockCount / config.blocksToKeep) * 100)),
            indexedHeight: maxHeight,
            blocksIndexed: blockCount,
          })

          height = batchEnd - 1
          yield* Effect.sleep(Duration.millis(50))
        }

        const [blockCount, txCount, maxHeight] = yield* Effect.all([
          db.getBlockCount(chain.id),
          db.getTxCount(chain.id),
          db.getLatestHeight(chain.id),
        ])
        yield* updateState(chain.id, {
          status: "synced",
          backfillProgress: 100,
          indexedHeight: maxHeight,
          blocksIndexed: blockCount,
          txsIndexed: txCount,
          lastSync: new Date().toISOString(),
        })
        yield* Effect.log(`[${chain.id}] Backfill done: ${blockCount} blocks, ${txCount} txs`)
      }).pipe(
        Effect.catchAll((e) =>
          updateState(chain.id, { status: "error", lastError: String(e) }).pipe(
            Effect.tap(() => Effect.log(`[${chain.id}] Error: ${e}`))
          )
        )
      )

    // Sync chain (incremental)
    const syncChain = (chain: ChainConfig) =>
      Effect.gen(function* () {
        const state = (yield* Ref.get(stateRef))[chain.id]
        if (state.status === "backfilling") return

        yield* updateState(chain.id, { status: "syncing" })
        const rpc = createRpcClient(chain)

        const status = yield* rpc.getStatus()
        const latestHeight = parseInt(status.result.sync_info.latest_block_height)
        const currentMax = yield* db.getLatestHeight(chain.id)

        yield* updateState(chain.id, { latestHeight })

        if (currentMax >= latestHeight) {
          yield* updateState(chain.id, { status: "synced", indexedHeight: currentMax, lastSync: new Date().toISOString() })
          return
        }

        // Fetch and insert per batch to avoid memory accumulation
        let height = currentMax + 1

        while (height <= latestHeight) {
          const batchEnd = Math.min(latestHeight, height + config.backfillBatchSize - 1)
          const heights = Array.from({ length: batchEnd - height + 1 }, (_, i) => height + i)

          const { blocks, txs } = yield* fetchBlockBatch(rpc, chain.id, heights)
          yield* db.insertBlocks(blocks)
          yield* db.insertTransactions(txs)
          height = batchEnd + 1
        }

        yield* db.pruneOldData(chain.id, latestHeight - config.blocksToKeep)

        const [blockCount, txCount] = yield* Effect.all([
          db.getBlockCount(chain.id),
          db.getTxCount(chain.id),
        ])
        yield* updateState(chain.id, {
          status: "synced",
          indexedHeight: latestHeight,
          blocksIndexed: blockCount,
          txsIndexed: txCount,
          lastSync: new Date().toISOString(),
        })
      }).pipe(
        Effect.catchAll((e) =>
          updateState(chain.id, { status: "error", lastError: String(e) })
        )
      )

    // Fetch and store chain stats (supply, staking)
    const fetchChainStats = (chain: ChainConfig) =>
      Effect.gen(function* () {
        const rpc = createRpcClient(chain)

        // Fetch all stats in parallel
        const [poolResult, supplyResult, inflationResult, communityPoolResult, statusResult] = yield* Effect.all([
          rpc.getStakingPool().pipe(Effect.catchAll(() => Effect.succeed(null))),
          rpc.getSupply().pipe(Effect.catchAll(() => Effect.succeed(null))),
          rpc.getInflation().pipe(Effect.catchAll(() => Effect.succeed(null))),
          rpc.getCommunityPool().pipe(Effect.catchAll(() => Effect.succeed(null))),
          rpc.getStatus().pipe(Effect.catchAll(() => Effect.succeed(null))),
        ])

        if (!statusResult || !poolResult) return

        const height = parseInt(statusResult.result.sync_info.latest_block_height)
        const totalSupply = supplyResult?.supply[0]?.amount ?? "0"
        const communityPool = communityPoolResult?.pool[0]?.amount ?? "0"

        yield* db.insertChainStats({
          chain_id: chain.id,
          height,
          timestamp: new Date().toISOString(),
          total_supply: totalSupply,
          bonded_tokens: poolResult.pool.bonded_tokens,
          not_bonded_tokens: poolResult.pool.not_bonded_tokens,
          inflation: inflationResult?.inflation ?? "0",
          community_pool: communityPool,
        })
      }).pipe(
        Effect.catchAll((e) => Effect.log(`[${chain.id}] Stats error: ${e}`))
      )

    return {
      getState: () => Ref.get(stateRef).pipe(Effect.map((chains) => ({ startedAt, chains }))),

      getChainState: (chainId) => Ref.get(stateRef).pipe(Effect.map((s) => s[chainId] || null)),

      start: () =>
        Effect.gen(function* () {
          // Start chain stats polling immediately (runs every 30 seconds)
          yield* Effect.log("Starting chain stats polling...")
          const statsPoll = Effect.all(CHAINS.map(fetchChainStats), { concurrency: "unbounded" }).pipe(
            Effect.catchAll((e) => Effect.log(`Stats poll error: ${e}`))
          )
          yield* Effect.forever(statsPoll.pipe(Effect.delay(Duration.millis(30_000)))).pipe(
            Effect.forkDaemon
          )

          // Run initial stats fetch immediately
          yield* statsPoll

          // For each chain: backfill then start its own sync loop
          const runChain = (chain: ChainConfig) =>
            Effect.gen(function* () {
              yield* backfillChain(chain)
              yield* Effect.log(`[${chain.id}] Starting sync loop...`)
              // Start sync loop for this chain
              yield* Effect.forever(
                syncChain(chain).pipe(
                  Effect.catchAll((e) => Effect.log(`[${chain.id}] Sync error: ${e}`)),
                  Effect.delay(Duration.millis(config.pollInterval))
                )
              )
            }).pipe(Effect.forkDaemon)

          yield* Effect.log("Starting chains...")
          yield* Effect.all(CHAINS.map(runChain), { concurrency: "unbounded" })
        }),
    }
  })
)
