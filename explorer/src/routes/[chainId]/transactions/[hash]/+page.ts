import { indexer } from "$lib/services/indexer-client"
import type { PageLoad } from "./$types"

export const prerender = false

export const load: PageLoad = async ({ params, parent }) => {
  const { hash } = params
  const { chainId, chain } = await parent()

  // Transform indexer tx to TxResponse format expected by the page
  const transaction = indexer.txByHash(chainId, hash).then(tx => {
    if (!tx) throw new Error(`Transaction ${hash} not found`)
    return {
      tx_response: {
        height: String(tx.height),
        txhash: tx.hash,
        code: tx.code,
        codespace: tx.codespace,
        data: "",
        raw_log: tx.raw_log,
        logs: [],
        info: "",
        gas_wanted: tx.gas_wanted,
        gas_used: tx.gas_used,
        tx: {
          "@type": "/cosmos.tx.v1beta1.Tx",
          body: {
            messages: tx.messages as Array<{ "@type": string }>,
            memo: tx.memo,
            timeout_height: "0",
            extension_options: [],
            non_critical_extension_options: [],
          },
          auth_info: {
            signer_infos: [],
            fee: tx.fee as { amount: Array<{ denom: string; amount: string }>; gas_limit: string; payer: string; granter: string },
          },
          signatures: [],
        },
        timestamp: tx.timestamp,
        events: tx.events as Array<{ type: string; attributes: Array<{ key: string; value: string }> }>,
      }
    }
  })

  return {
    hash,
    transaction,
    chain,
  }
}
