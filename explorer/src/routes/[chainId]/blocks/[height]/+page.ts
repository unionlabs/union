import { createChainRuntime } from "$lib/runtime"
import { fetchBlockByHeight, fetchValidatorSet } from "$lib/queries/blocks"
import { fetchValidators } from "$lib/queries/validators"
import { indexer } from "$lib/services/indexer-client"
import type { PageLoad } from "./$types"

export const prerender = false

export const load: PageLoad = async ({ params, parent }) => {
  const { height } = params
  const { chainId, chain } = await parent()
  const runtime = createChainRuntime(chainId)

  // Use RPC for block (full nested structure), indexer for transactions
  return {
    height,
    block: runtime.runPromise(fetchBlockByHeight(height)),
    transactions: indexer.txsByHeight(chainId, parseInt(height)).then(txs => ({
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
    })),
    validatorSet: runtime.runPromise(fetchValidatorSet(height)),
    validators: runtime.runPromise(fetchValidators("BOND_STATUS_BONDED")),
    chain,
  }
}
