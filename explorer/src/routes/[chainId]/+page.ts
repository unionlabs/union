import { createChainRuntime } from "$lib/runtime"
import { fetchLatestBlock } from "$lib/queries/blocks"
import { fetchValidators } from "$lib/queries/validators"
import { fetchProposals } from "$lib/queries/governance"
import { indexer } from "$lib/services/indexer-client"
import type { PageLoad } from "./$types"

export const load: PageLoad = async ({ depends, parent }) => {
  depends("home:data")

  const { chainId, chain } = await parent()
  const runtime = createChainRuntime(chainId)

  // Use indexer for recent blocks (faster than RPC)
  const recentBlocks = indexer.blocks(chainId, 10).then(blocks =>
    blocks.map(b => ({
      height: String(b.height),
      hash: b.hash,
      time: b.time,
      proposer: b.proposer,
      txCount: b.tx_count,
    }))
  ).catch(() => [])

  // Use indexer for recent transactions (much faster than RPC scanning)
  const recentTxs = indexer.txs(chainId, 10).then(txs => ({
    tx_responses: txs.map(tx => ({
      height: String(tx.height),
      txhash: tx.hash,
      code: tx.code,
      codespace: tx.codespace,
      raw_log: tx.raw_log,
      gas_wanted: tx.gas_wanted,
      gas_used: tx.gas_used,
      tx: {
        body: {
          messages: tx.messages,
          memo: tx.memo,
        },
        auth_info: {
          fee: tx.fee,
        },
      },
      timestamp: tx.timestamp,
      events: tx.events,
    })),
    pagination: { total: String(txs.length), next_key: null },
  })).catch(() => ({
    tx_responses: [],
    pagination: { total: "0", next_key: null },
  }))

  // Use indexer for chain stats (supply, staking) - much faster than RPC
  const chainStats = indexer.chainStats(chainId).catch(() => null)

  return {
    latestBlock: runtime.runPromise(fetchLatestBlock()),
    recentBlocks,
    recentTxs,
    validators: runtime.runPromise(fetchValidators("BOND_STATUS_BONDED")),
    chainStats,
    proposals: runtime.runPromise(fetchProposals()),
    chain,
  }
}
