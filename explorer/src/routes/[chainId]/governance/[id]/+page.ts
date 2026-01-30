import { createChainRuntime } from "$lib/runtime"
import { fetchProposal, fetchProposalVotes, fetchProposalTally } from "$lib/queries/governance"
import { fetchStakingPool, fetchValidators } from "$lib/queries/validators"
import type { PageLoad } from "./$types"

export const prerender = false

export const load: PageLoad = async ({ params, depends, parent }) => {
  depends("proposal:data")

  const { chainId, chain } = await parent()
  const runtime = createChainRuntime(chainId)

  return {
    id: params.id,
    proposal: runtime.runPromise(fetchProposal(params.id)),
    votes: runtime.runPromise(fetchProposalVotes(params.id)),
    tally: runtime.runPromise(fetchProposalTally(params.id)),
    stakingPool: runtime.runPromise(fetchStakingPool()),
    validators: runtime.runPromise(fetchValidators("BOND_STATUS_BONDED")),
    chain,
  }
}
