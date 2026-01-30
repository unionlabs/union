// Simple chain API client - all requests go through indexer proxy
// Indexer handles: serving indexed data or proxying to chain REST

import { INDEXER_URL } from "$lib/config"

const TIMEOUT = 15_000

export class ChainApiError extends Error {
  constructor(message: string, public status?: number) {
    super(message)
  }
}

async function fetchWithTimeout<T>(url: string): Promise<T> {
  const controller = new AbortController()
  const timeoutId = setTimeout(() => controller.abort(), TIMEOUT)

  try {
    const res = await fetch(url, { signal: controller.signal })

    if (!res.ok) {
      const body = await res.json().catch(() => ({}))
      throw new ChainApiError(body.error || `HTTP ${res.status}`, res.status)
    }

    return res.json()
  } catch (e) {
    if (e instanceof ChainApiError) {
      throw e
    }
    if (e instanceof Error && e.name === "AbortError") {
      throw new ChainApiError("Request timeout")
    }
    throw new ChainApiError(e instanceof Error ? e.message : String(e))
  } finally {
    clearTimeout(timeoutId)
  }
}

// REST API requests go through /rest proxy
async function fetchRest<T>(chainId: string, path: string): Promise<T> {
  const url = `${INDEXER_URL}/${encodeURIComponent(chainId)}/rest${path}`
  return fetchWithTimeout<T>(url)
}

// RPC API requests go through /rpc proxy
async function fetchRpc<T>(chainId: string, path: string): Promise<T> {
  const url = `${INDEXER_URL}/${encodeURIComponent(chainId)}/rpc/${path}`
  return fetchWithTimeout<T>(url)
}

// RPC endpoints (via RPC proxy)
export const rpc = {
  status: (chainId: string) =>
    fetchRpc<{ result: { sync_info: { latest_block_height: string } } }>(chainId, "status"),

  blockchain: (chainId: string, minHeight: number, maxHeight: number) =>
    fetchRpc<{
      result: {
        block_metas: Array<{
          block_id: { hash: string }
          header: { height: string; time: string; proposer_address: string }
          num_txs: string
        }>
      }
    }>(chainId, `blockchain?minHeight=${minHeight}&maxHeight=${maxHeight}`),

  txSearch: (chainId: string, query: string, page = 1, perPage = 50) =>
    fetchRpc<{
      result: {
        txs: Array<{
          hash: string
          height: string
          tx_result: {
            code: number
            gas_wanted: string
            gas_used: string
            events: Array<{ type: string; attributes: Array<{ key: string; value: string }> }>
          }
        }>
        total_count: string
      }
    }>(
      chainId,
      `tx_search?query="${
        encodeURIComponent(query)
      }"&page=${page}&per_page=${perPage}&order_by="desc"`,
    ),
}

// REST endpoints (via REST proxy)
export const rest = {
  latestBlock: (chainId: string) =>
    fetchRest<{
      block_id: { hash: string }
      block: {
        header: { height: string; time: string; proposer_address: string }
        data: { txs: string[] | null }
      }
    }>(chainId, "/cosmos/base/tendermint/v1beta1/blocks/latest"),

  block: (chainId: string, height: string) =>
    fetchRest<{
      block_id: { hash: string }
      block: {
        header: { height: string; time: string; proposer_address: string }
        data: { txs: string[] | null }
      }
    }>(chainId, `/cosmos/base/tendermint/v1beta1/blocks/${height}`),

  tx: (chainId: string, hash: string) =>
    fetchRest<{ tx_response: TxResponse }>(chainId, `/cosmos/tx/v1beta1/txs/${hash}`),

  validators: (chainId: string, status?: string) => {
    const params = status ? `?status=${status}&pagination.limit=500` : "?pagination.limit=500"
    return fetchRest<{ validators: Validator[]; pagination: unknown }>(
      chainId,
      `/cosmos/staking/v1beta1/validators${params}`,
    )
  },

  validator: (chainId: string, address: string) =>
    fetchRest<{ validator: Validator }>(chainId, `/cosmos/staking/v1beta1/validators/${address}`),

  balances: (chainId: string, address: string) =>
    fetchRest<{ balances: Coin[] }>(chainId, `/cosmos/bank/v1beta1/balances/${address}`),

  delegations: (chainId: string, address: string) =>
    fetchRest<{ delegation_responses: Delegation[] }>(
      chainId,
      `/cosmos/staking/v1beta1/delegations/${address}`,
    ),

  proposals: (chainId: string) =>
    fetchRest<{ proposals: Proposal[] }>(
      chainId,
      "/cosmos/gov/v1/proposals?pagination.limit=100&pagination.reverse=true",
    ),

  proposal: (chainId: string, id: string) =>
    fetchRest<{ proposal: Proposal }>(chainId, `/cosmos/gov/v1/proposals/${id}`),

  stakingParams: (chainId: string) =>
    fetchRest<{ params: StakingParams }>(chainId, "/cosmos/staking/v1beta1/params"),

  stakingPool: (chainId: string) =>
    fetchRest<{ pool: StakingPool }>(chainId, "/cosmos/staking/v1beta1/pool"),

  nodeInfo: (chainId: string) =>
    fetchRest<{ default_node_info: { network: string } }>(
      chainId,
      "/cosmos/base/tendermint/v1beta1/node_info",
    ),
}

// Types (simplified)
export interface TxResponse {
  height: string
  txhash: string
  code: number
  raw_log: string
  gas_wanted: string
  gas_used: string
  tx: { body: { messages: unknown[] }; auth_info: { fee: unknown } }
  timestamp: string
  events: unknown[]
}

export interface Validator {
  operator_address: string
  consensus_pubkey: unknown
  jailed: boolean
  status: string
  tokens: string
  delegator_shares: string
  description: {
    moniker: string
    identity: string
    website: string
    details: string
  }
  commission: {
    commission_rates: { rate: string; max_rate: string; max_change_rate: string }
  }
}

export interface Coin {
  denom: string
  amount: string
}

export interface Delegation {
  delegation: { delegator_address: string; validator_address: string; shares: string }
  balance: Coin
}

export interface Proposal {
  id: string
  status: string
  submit_time: string
  voting_end_time: string
  title?: string
  summary?: string
  messages: unknown[]
  final_tally_result?: {
    yes_count: string
    no_count: string
    abstain_count: string
    no_with_veto_count: string
  }
}

export interface StakingParams {
  unbonding_time: string
  max_validators: number
  bond_denom: string
}

export interface StakingPool {
  bonded_tokens: string
  not_bonded_tokens: string
}
