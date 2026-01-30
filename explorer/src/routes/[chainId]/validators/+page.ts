import { createChainRuntime } from "$lib/runtime"
import { fetchValidators, fetchStakingPool, fetchStakingParams } from "$lib/queries/validators"
import type { PageLoad } from "./$types"

export const load: PageLoad = async ({ depends, parent }) => {
  depends("validators:data")

  const { chainId, chain } = await parent()
  const runtime = createChainRuntime(chainId)

  return {
    validators: runtime.runPromise(fetchValidators("BOND_STATUS_BONDED")),
    stakingPool: runtime.runPromise(fetchStakingPool()),
    stakingParams: runtime.runPromise(fetchStakingParams()),
    chain,
  }
}
