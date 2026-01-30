import { Context, Duration, Effect, Layer, Ref, Schedule } from "effect"
import { type ChainConfig, CHAINS, IndexerConfigService } from "./config.js"
import { type Block, Database, type Transaction } from "./Db.js"
import { DatabaseError, UpstreamError } from "./errors.js"
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
    tx_hashes: [],
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
    tx_bytes: "",
    timestamp: tx.timestamp || "",
  }
}

// ============ Implementation ============

export const SyncLive = Layer.effect(
  Sync,
  Effect.gen(function*() {
    const db = yield* Database
    const config = yield* IndexerConfigService

    const startedAt = new Date().toISOString()

    // Initialize state for all chains
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

    // Atomic state update - uses Ref.modify for atomicity
    const updateState = (chainId: string, update: Partial<ChainSyncState>) =>
      Ref.modify(stateRef, (s) => {
        const current = s[chainId]
        if (!current) {
          return [undefined, s]
        }
        const next = { ...current, ...update }
        return [next, { ...s, [chainId]: next }]
      })

    // Get current state for a chain atomically
    const getChainStateAtomic = (chainId: string) =>
      Ref.get(stateRef).pipe(Effect.map((s) => s[chainId] ?? null))

    // Fetch full block with all txs - returns typed error on failure
    const fetchFullBlockWithTxs = (
      rpc: ReturnType<typeof createRpcClient>,
      chainId: string,
      height: number,
    ): Effect.Effect<
      { block: Block | null; txs: Transaction[] },
      UpstreamError | DatabaseError
    > =>
      Effect.gen(function*() {
        const restBlock = yield* rpc.getFullBlock(height).pipe(
          Effect.catchAll((e) =>
            Effect.gen(function*() {
              yield* Effect.logWarning(`[${chainId}] Block ${height} fetch failed: ${e.message}`)
              return null
            })
          ),
        )

        if (!restBlock) {
          return { block: null, txs: [] }
        }

        const block = restBlockToBlock(chainId, restBlock)
        const txs: Transaction[] = []

        if (block.tx_count > 0) {
          const txResponse = yield* rpc.getTxsByHeight(height).pipe(
            Effect.catchAll((e) =>
              Effect.gen(function*() {
                yield* Effect.logWarning(
                  `[${chainId}] Txs for block ${height} fetch failed: ${e.message}`,
                )
                return { tx_responses: [] as RestTxResponse[] }
              })
            ),
          )

          for (let i = 0; i < (txResponse.tx_responses?.length ?? 0); i++) {
            const txResp = txResponse.tx_responses[i]
            if (txResp) {
              txs.push(restTxToTransaction(chainId, txResp, i))
            }
          }
          block.tx_hashes = txs.map((t) => t.hash)
        }

        return { block, txs }
      })

    // Batch fetch with concurrency limit
    const fetchBlockBatch = (
      rpc: ReturnType<typeof createRpcClient>,
      chainId: string,
      heights: number[],
    ) =>
      Effect.all(
        heights.map((h) => fetchFullBlockWithTxs(rpc, chainId, h)),
        { concurrency: 5 },
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
        }),
      )

    // Backfill chain with proper error handling
    const backfillChain = (
      chain: ChainConfig,
    ): Effect.Effect<void, UpstreamError | DatabaseError> =>
      Effect.gen(function*() {
        yield* updateState(chain.id, { status: "backfilling", lastError: null })
        const rpc = createRpcClient(chain)

        const status = yield* rpc.getStatus()
        const latestHeight = parseInt(status.result.sync_info.latest_block_height)
        const targetMin = Math.max(1, latestHeight - config.blocksToKeep + 1)
        const currentMax = yield* db.getLatestHeight(chain.id)

        yield* updateState(chain.id, { latestHeight })

        if (currentMax >= latestHeight - 10) {
          yield* updateState(chain.id, {
            status: "synced",
            backfillProgress: 100,
            indexedHeight: currentMax,
          })
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

          // Insert blocks and transactions atomically
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
      })

    // Sync chain (incremental) with proper error handling
    const syncChain = (
      chain: ChainConfig,
    ): Effect.Effect<void, UpstreamError | DatabaseError> =>
      Effect.gen(function*() {
        // Check status atomically
        const state = yield* getChainStateAtomic(chain.id)
        if (!state || state.status === "backfilling") {
          return
        }

        yield* updateState(chain.id, { status: "syncing", lastError: null })
        const rpc = createRpcClient(chain)

        const status = yield* rpc.getStatus()
        const latestHeight = parseInt(status.result.sync_info.latest_block_height)
        const currentMax = yield* db.getLatestHeight(chain.id)

        yield* updateState(chain.id, { latestHeight })

        if (currentMax >= latestHeight) {
          yield* updateState(chain.id, {
            status: "synced",
            indexedHeight: currentMax,
            lastSync: new Date().toISOString(),
          })
          return
        }

        // Fetch and insert per batch
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
      })

    // Fetch and store chain stats (supply, staking)
    const fetchChainStats = (chain: ChainConfig): Effect.Effect<void> =>
      Effect.gen(function*() {
        const rpc = createRpcClient(chain)

        // Fetch all stats in parallel, each with its own error handling
        const [poolResult, supplyResult, inflationResult, communityPoolResult, statusResult] =
          yield* Effect.all([
            rpc.getStakingPool().pipe(Effect.option),
            rpc.getSupply().pipe(Effect.option),
            rpc.getInflation().pipe(Effect.option),
            rpc.getCommunityPool().pipe(Effect.option),
            rpc.getStatus().pipe(Effect.option),
          ])

        // Skip if we don't have the minimum required data
        if (
          !statusResult._tag || statusResult._tag !== "Some" || !poolResult._tag
          || poolResult._tag !== "Some"
        ) {
          return
        }

        const statusValue = statusResult.value
        const poolValue = poolResult.value

        const height = parseInt(statusValue.result.sync_info.latest_block_height)
        const totalSupply = supplyResult._tag === "Some"
          ? (supplyResult.value.supply[0]?.amount ?? "0")
          : "0"
        const communityPool = communityPoolResult._tag === "Some"
          ? (communityPoolResult.value.pool[0]?.amount ?? "0")
          : "0"

        yield* db
          .insertChainStats({
            chain_id: chain.id,
            height,
            timestamp: new Date().toISOString(),
            total_supply: totalSupply,
            bonded_tokens: poolValue.pool.bonded_tokens,
            not_bonded_tokens: poolValue.pool.not_bonded_tokens,
            inflation: inflationResult._tag === "Some" ? inflationResult.value.inflation : "0",
            community_pool: communityPool,
          })
          .pipe(Effect.catchAll(() => Effect.void))
      }).pipe(Effect.catchAll((e) => Effect.logWarning(`[${chain.id}] Stats error: ${e}`)))

    // Supervised chain sync with restart on failure
    const supervisedChainLoop = (chain: ChainConfig) => {
      // Retry policy for the sync loop: exponential backoff with max 5 retries before giving up
      const syncRetryPolicy = Schedule.exponential(Duration.seconds(5)).pipe(
        Schedule.intersect(Schedule.recurs(5)),
        Schedule.jittered,
      )

      const singleSyncWithRecovery = syncChain(chain).pipe(
        Effect.catchAll((e) =>
          Effect.gen(function*() {
            const errorMsg = e instanceof Error ? e.message : String(e)
            yield* updateState(chain.id, { status: "error", lastError: errorMsg })
            yield* Effect.logError(`[${chain.id}] Sync error: ${errorMsg}`)
            // Re-throw to trigger retry
            return yield* Effect.fail(e)
          })
        ),
        Effect.retry(syncRetryPolicy),
        Effect.catchAll((_e) =>
          Effect.gen(function*() {
            yield* Effect.logError(`[${chain.id}] Sync failed after retries, will try again later`)
            yield* updateState(chain.id, { status: "error", lastError: "Max retries exceeded" })
          })
        ),
      )

      return Effect.forever(
        singleSyncWithRecovery.pipe(Effect.delay(Duration.millis(config.pollInterval))),
      )
    }

    // Run chain with backfill then supervised sync loop
    const runChain = (chain: ChainConfig) =>
      Effect.gen(function*() {
        // Backfill with error recovery
        yield* backfillChain(chain).pipe(
          Effect.catchAll((e) =>
            Effect.gen(function*() {
              const errorMsg = e instanceof Error ? e.message : String(e)
              yield* updateState(chain.id, { status: "error", lastError: errorMsg })
              yield* Effect.logError(`[${chain.id}] Backfill error: ${errorMsg}`)
            })
          ),
        )

        yield* Effect.log(`[${chain.id}] Starting supervised sync loop...`)

        // Start supervised sync loop as daemon
        yield* supervisedChainLoop(chain).pipe(Effect.forkDaemon)
      })

    return {
      getState: () => Ref.get(stateRef).pipe(Effect.map((chains) => ({ startedAt, chains }))),

      getChainState: (chainId) => getChainStateAtomic(chainId),

      start: () =>
        Effect.gen(function*() {
          // Start chain stats polling immediately (runs every 30 seconds)
          yield* Effect.log("Starting chain stats polling...")
          const statsPoll = Effect.all(CHAINS.map(fetchChainStats), { concurrency: 3 }).pipe(
            Effect.catchAll((e) => Effect.logWarning(`Stats poll error: ${e}`)),
          )

          yield* Effect.forever(statsPoll.pipe(Effect.delay(Duration.millis(30_000)))).pipe(
            Effect.forkDaemon,
          )

          // Run initial stats fetch immediately
          yield* statsPoll

          // Start each chain with controlled concurrency (max 3 chains initializing at once)
          yield* Effect.log("Starting chains...")
          yield* Effect.all(CHAINS.map(runChain), { concurrency: 3 })
        }),
    }
  }),
)
