import { derived, type Readable } from "svelte/store"
import type { Chain, UserAddresses } from "$lib/types"
import { userAddress } from "./balances.ts"
import type { BalanceData, userBalancesQuery } from "$lib/queries/balance"
import type { UnwrapReadable } from "$lib/utilities/types.ts"

export type ChainBalances = {
  chainId: string
  balances: Array<BalanceData>
}

export type BalancesList = Array<ChainBalances>

export interface ContextStore {
  chains: Array<Chain>
  userAddress: UserAddresses
  balances: UnwrapReadable<ReturnType<typeof userBalancesQuery>>
}

export function createContextStore(
  chains: Array<Chain>,
  balances: ReturnType<typeof userBalancesQuery>
): Readable<ContextStore> {
  return derived([userAddress, balances], ([$userAddress, $balances]) => ({
    chains,
    userAddress: $userAddress,
    balances: $balances
  }))
}
