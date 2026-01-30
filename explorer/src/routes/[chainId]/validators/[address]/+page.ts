import { createChainRuntime } from "$lib/runtime"
import { fetchValidator, fetchValidatorDelegations, fetchStakingPool } from "$lib/queries/validators"
import { indexer } from "$lib/services/indexer-client"
import type { PageLoad } from "./$types"

export const prerender = false

export const load: PageLoad = async ({ params, depends, parent }) => {
  depends("validator:data")

  const { chainId, chain } = await parent()
  const runtime = createChainRuntime(chainId)

  // Use indexer for blocks with signatures (much faster than 100 RPC calls)
  const recentBlocks = indexer.blocks(chainId, 100).then(blocks =>
    blocks.map(b => ({
      height: String(b.height),
      hash: b.hash,
      time: b.time,
      proposer: b.proposer,
      txCount: b.tx_count,
      header: b.header,
      signatures: b.signatures,
      txHashes: b.tx_hashes,
    }))
  ).catch(() => [])

  return {
    address: params.address,
    validator: runtime.runPromise(fetchValidator(params.address)),
    delegations: runtime.runPromise(fetchValidatorDelegations(params.address)),
    stakingPool: runtime.runPromise(fetchStakingPool()),
    recentBlocks,
    chain,
  }
}
