import { Duration, Effect, Schedule } from "effect"
import type { ChainConfig } from "./config.js"

// ============ RPC Response Types ============

export interface StatusResponse {
  result: {
    sync_info: {
      latest_block_height: string
    }
  }
}

export interface BlockMeta {
  block_id: { hash: string }
  header: { height: string; time: string; proposer_address: string }
  num_txs: string
}

export interface BlockchainResponse {
  result: { block_metas: BlockMeta[] }
}

// ============ REST Response Types ============

// Full block from REST API
export interface RestBlock {
  block_id: {
    hash: string
    part_set_header: { total: number; hash: string }
  }
  block: {
    header: {
      version: { block: string; app?: string }
      chain_id: string
      height: string
      time: string
      last_block_id: { hash: string; part_set_header: { total: number; hash: string } }
      last_commit_hash: string
      data_hash: string
      validators_hash: string
      next_validators_hash: string
      consensus_hash: string
      app_hash: string
      last_results_hash: string
      evidence_hash: string
      proposer_address: string
    }
    data: {
      txs: string[] | null // Base64 encoded txs
    }
    evidence: { evidence: unknown[] }
    last_commit: {
      height: string
      round: number
      block_id: { hash: string; part_set_header: { total: number; hash: string } }
      signatures: Array<{
        block_id_flag: string
        validator_address: string
        timestamp: string
        signature: string | null
      }>
    } | null
  }
}

// Full transaction from REST API
export interface RestTxResponse {
  height: string
  txhash: string
  codespace: string
  code: number
  data: string
  raw_log: string
  logs: unknown[]
  info: string
  gas_wanted: string
  gas_used: string
  tx: {
    "@type": string
    "body": {
      messages: unknown[]
      memo: string
      timeout_height: string
      extension_options: unknown[]
      non_critical_extension_options: unknown[]
    }
    "auth_info": {
      signer_infos: unknown[]
      fee: {
        amount: Array<{ denom: string; amount: string }>
        gas_limit: string
        payer: string
        granter: string
      }
    }
    "signatures": string[]
  }
  timestamp: string
  events: Array<
    { type: string; attributes: Array<{ key: string; value: string; index?: boolean }> }
  >
}

export interface RestTxsResponse {
  txs: unknown[]
  tx_responses: RestTxResponse[]
  pagination: { total: string; next_key: string | null }
}

// Staking pool response
export interface StakingPoolResponse {
  pool: {
    not_bonded_tokens: string
    bonded_tokens: string
  }
}

// Supply response
export interface SupplyResponse {
  supply: Array<{ denom: string; amount: string }>
  pagination: { next_key: string | null; total: string }
}

// Inflation response
export interface InflationResponse {
  inflation: string
}

// Community pool response
export interface CommunityPoolResponse {
  pool: Array<{ denom: string; amount: string }>
}

// ============ Fetch helpers ============

const fetchRace = <T>(
  endpoints: string[],
  path: string,
  timeout = 15000,
): Effect.Effect<T, Error> =>
  Effect.gen(function*() {
    if (endpoints.length === 0) {
      return yield* Effect.fail(new Error("No endpoints"))
    }

    const controller = new AbortController()
    const timeoutId = setTimeout(() => controller.abort(), timeout)

    try {
      const result = yield* Effect.tryPromise({
        try: () =>
          Promise.any(
            endpoints.map(async (base) => {
              const url = `${base.replace(/\/$/, "")}${path}`
              const res = await fetch(url, { signal: controller.signal })
              if (!res.ok) {
                throw new Error(`HTTP ${res.status}`)
              }
              return res.json()
            }),
          ),
        catch: (e) => new Error(String(e)),
      })
      return result as T
    } finally {
      clearTimeout(timeoutId)
    }
  })

// ============ Retry Policy ============

// Exponential backoff: 100ms, 200ms, 400ms with jitter, max 3 retries
const retryPolicy = Schedule.intersect(
  Schedule.exponential(Duration.millis(100)),
  Schedule.recurs(3),
).pipe(Schedule.jittered)

// ============ RPC Client ============

export const createRpcClient = (chain: ChainConfig) => ({
  // RPC endpoints
  getStatus: () => fetchRace<StatusResponse>(chain.rpc, "/status").pipe(Effect.retry(retryPolicy)),

  getBlockchain: (min: number, max: number) =>
    fetchRace<BlockchainResponse>(chain.rpc, `/blockchain?minHeight=${min}&maxHeight=${max}`).pipe(
      Effect.retry(retryPolicy),
    ),

  // REST endpoints for full data
  getFullBlock: (height: number) =>
    fetchRace<RestBlock>(chain.rest, `/cosmos/base/tendermint/v1beta1/blocks/${height}`).pipe(
      Effect.retry(retryPolicy),
    ),

  getTxsByHeight: (height: number) =>
    fetchRace<RestTxsResponse>(
      chain.rest,
      `/cosmos/tx/v1beta1/txs?query=tx.height=${height}&pagination.limit=100`,
    ).pipe(
      Effect.retry(retryPolicy),
    ),

  // Chain stats endpoints
  getStakingPool: () =>
    fetchRace<StakingPoolResponse>(chain.rest, `/cosmos/staking/v1beta1/pool`).pipe(
      Effect.retry(retryPolicy),
    ),

  getSupply: () =>
    fetchRace<SupplyResponse>(chain.rest, `/cosmos/bank/v1beta1/supply?pagination.limit=1`).pipe(
      Effect.retry(retryPolicy),
    ),

  getInflation: () =>
    fetchRace<InflationResponse>(chain.rest, `/cosmos/mint/v1beta1/inflation`).pipe(
      Effect.retry(retryPolicy),
    ),

  getCommunityPool: () =>
    fetchRace<CommunityPoolResponse>(chain.rest, `/cosmos/distribution/v1beta1/community_pool`)
      .pipe(
        Effect.retry(retryPolicy),
      ),
})

// ============ Type exports for Sync ============

export type RpcClient = ReturnType<typeof createRpcClient>
