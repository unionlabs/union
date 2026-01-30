import { createChainRuntime } from "$lib/runtime"
import {
  fetchAccount,
  fetchBalances,
  fetchDelegations,
  fetchUnbondingDelegations,
  fetchDelegatorRewards,
  fetchAllAccountTxs,
} from "$lib/queries/accounts"
import { fetchValidators } from "$lib/queries/validators"
import type { PageLoad } from "./$types"

export const prerender = false

export const load: PageLoad = async ({ params, depends, parent }) => {
  depends("account:data")

  const { chainId, chain } = await parent()
  const runtime = createChainRuntime(chainId)

  return {
    address: params.address,
    account: runtime.runPromise(fetchAccount(params.address)),
    balances: runtime.runPromise(fetchBalances(params.address)),
    delegations: runtime.runPromise(fetchDelegations(params.address)),
    unbonding: runtime.runPromise(fetchUnbondingDelegations(params.address)),
    rewards: runtime.runPromise(fetchDelegatorRewards(params.address)),
    transactions: runtime.runPromise(fetchAllAccountTxs(params.address, 1, 25)),
    validators: runtime.runPromise(fetchValidators("BOND_STATUS_BONDED")),
    chain,
  }
}
