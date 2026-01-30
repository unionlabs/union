/**
 * Client for the indexer-v2 service (Effect + SQLite)
 */
import { INDEXER_URL } from "$lib/config"

// ============ Types matching indexer-v2 API ============

export interface IndexedBlock {
  chain_id: string
  height: number
  hash: string
  time: string
  proposer: string
  tx_count: number
  header: BlockHeader
  signatures: CommitSignature[]
  tx_hashes: string[]
}

export interface BlockHeader {
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

export interface CommitSignature {
  block_id_flag: string
  validator_address: string
  timestamp: string
  signature: string | null
}

export interface IndexedTx {
  chain_id: string
  hash: string
  height: number
  index: number
  code: number
  codespace: string
  gas_used: string
  gas_wanted: string
  messages: TxMessage[]
  memo: string
  fee: TxFee
  events: TxEvent[]
  raw_log: string
  tx_bytes: string
  timestamp: string
}

export interface TxMessage {
  "@type": string
  [key: string]: unknown
}

export interface TxFee {
  amount: Array<{ denom: string; amount: string }>
  gas_limit: string
  payer: string
  granter: string
}

export interface TxEvent {
  type: string
  attributes: Array<{ key: string; value: string; index?: boolean }>
}

export interface ChainStatus {
  chain_id: string
  name: string
  status: "idle" | "backfilling" | "syncing" | "synced" | "error"
  latestHeight: number
  indexedHeight: number
  blocksIndexed: number
  txsIndexed: number
  backfillProgress: number
  lastSync: string | null
  lastError: string | null
}

export interface ChainInfo {
  id: string
  name: string
}

export interface HealthResponse {
  status: "healthy" | "syncing" | "degraded"
  started_at: string
  uptime_seconds: number
  db_size_bytes: number
  chains: ChainStatus[]
}

export interface MsgTypeStats {
  msg_type: string | null
  count: number
  success_count: number
  failure_count: number
}

export interface DailyStats {
  date: string
  block_count: number
  tx_count: number
}

export interface HourlyStats {
  hour: string
  count: number
}

export interface ChainStats {
  chain_id: string
  height: number
  timestamp: string
  total_supply: string
  bonded_tokens: string
  not_bonded_tokens: string
  inflation: string
  community_pool: string
}

// ============ Fetch helper ============

async function fetchJson<T>(path: string): Promise<T> {
  const res = await fetch(`${INDEXER_URL}${path}`)
  if (!res.ok) {
    throw new Error(`Indexer error: ${res.status}`)
  }
  return res.json()
}

// ============ API Client ============

export const indexer = {
  // Health & chains
  async health(): Promise<HealthResponse> {
    return fetchJson("/health")
  },

  async chains(): Promise<ChainInfo[]> {
    return fetchJson("/chains")
  },

  // Blocks
  async blocks(chainId: string, limit = 50, before?: number): Promise<IndexedBlock[]> {
    const params = new URLSearchParams({ limit: String(limit) })
    if (before !== undefined) {
      params.set("before", String(before))
    }
    const data = await fetchJson<{ blocks: IndexedBlock[] }>(`/${chainId}/blocks?${params}`)
    return data.blocks
  },

  async blockByHeight(chainId: string, height: number): Promise<IndexedBlock | null> {
    const data = await fetchJson<{ block: IndexedBlock | null }>(`/${chainId}/block/${height}`)
    return data.block
  },

  // Transactions
  async txs(chainId: string, limit = 50, before?: number): Promise<IndexedTx[]> {
    const params = new URLSearchParams({ limit: String(limit) })
    if (before !== undefined) {
      params.set("before", String(before))
    }
    const data = await fetchJson<{ txs: IndexedTx[] }>(`/${chainId}/txs?${params}`)
    return data.txs
  },

  async txByHash(chainId: string, hash: string): Promise<IndexedTx | null> {
    const data = await fetchJson<{ tx: IndexedTx | null }>(`/${chainId}/tx/${hash}`)
    return data.tx
  },

  async txsByHeight(chainId: string, height: number): Promise<IndexedTx[]> {
    const data = await fetchJson<{ txs: IndexedTx[] }>(`/${chainId}/block/${height}/txs`)
    return data.txs
  },

  async txsByAddress(chainId: string, address: string, limit = 50): Promise<IndexedTx[]> {
    const data = await fetchJson<{ txs: IndexedTx[] }>(
      `/${chainId}/address/${address}/txs?limit=${limit}`,
    )
    return data.txs
  },

  // Chain status
  async status(chainId: string): Promise<ChainStatus> {
    return fetchJson(`/${chainId}/status`)
  },

  // Analytics
  async msgTypeStats(chainId: string): Promise<MsgTypeStats[]> {
    const data = await fetchJson<{ stats: MsgTypeStats[] }>(`/${chainId}/analytics/msg-types`)
    return data.stats
  },

  async dailyStats(chainId: string, days = 7): Promise<DailyStats[]> {
    const data = await fetchJson<{ stats: DailyStats[] }>(
      `/${chainId}/analytics/daily?days=${days}`,
    )
    return data.stats
  },

  async hourlyStats(chainId: string, hours = 24): Promise<HourlyStats[]> {
    const data = await fetchJson<{ stats: HourlyStats[] }>(
      `/${chainId}/analytics/hourly?hours=${hours}`,
    )
    return data.stats
  },

  // Chain stats (supply, staking)
  async chainStats(chainId: string): Promise<ChainStats | null> {
    const data = await fetchJson<{ stats: ChainStats | null }>(`/${chainId}/analytics/stats`)
    return data.stats
  },

  async chainStatsHistory(chainId: string, limit = 100): Promise<ChainStats[]> {
    const data = await fetchJson<{ stats: ChainStats[] }>(
      `/${chainId}/analytics/stats/history?limit=${limit}`,
    )
    return data.stats
  },
}
