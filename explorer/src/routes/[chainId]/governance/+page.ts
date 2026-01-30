import { createChainRuntime } from "$lib/runtime"
import { fetchProposals } from "$lib/queries/governance"
import { fetchStakingPool } from "$lib/queries/validators"
import type { PageLoad } from "./$types"

export const prerender = false

export const load: PageLoad = async ({ depends, parent }) => {
  depends("governance:data")

  const { chainId, chain } = await parent()
  const runtime = createChainRuntime(chainId)

  return {
    proposals: runtime.runPromise(fetchProposals()),
    stakingPool: runtime.runPromise(fetchStakingPool()),
    chain,
  }
}
