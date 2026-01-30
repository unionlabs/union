import { indexer, type IndexedTx } from "$lib/services/indexer-client"
import type { PageLoad } from "./$types"

// Convert indexed tx to TxResponse format for UI compatibility
// Now using full data from indexer-v2
function toTxResponse(tx: IndexedTx) {
  return {
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
    // Additional fields from indexer
    index: tx.index,
  }
}

export const load: PageLoad = async ({ depends, parent }) => {
  depends("txs:data")

  const { chainId, chain } = await parent()

  async function getTransactions() {
    const txs = await indexer.txs(chainId, 50)
    return {
      tx_responses: txs.map(toTxResponse),
      pagination: { total: String(txs.length), next_key: null },
    }
  }

  return {
    transactions: getTransactions(),
    chain,
  }
}
