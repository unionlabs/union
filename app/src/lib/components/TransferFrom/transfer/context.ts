import { derived, type Readable } from "svelte/store"
import type { Chain, UserAddresses } from "$lib/types"
import { balanceStore, userAddress } from "./balances.ts"
import type { BalanceData } from "$lib/queries/balance"

export type ChainBalances = {
  chainId: string
  balances: Array<BalanceData>
}

export type BalancesList = Array<ChainBalances>

export interface ContextStore {
  chains: Array<Chain>
  userAddress: UserAddresses
  balances: BalancesList
}

export function createContextStore(chains: Array<Chain>): Readable<ContextStore> {
  const balances = derived(
    balanceStore,
    ($rawBalances: BalanceData[][]) => {
      if ($rawBalances?.length === 0) {
        return chains.map(chain => ({
          chainId: chain.chain_id,
          balances: []
        }))
      }

      return chains.map((chain, chainIndex) => {
        const chainBalances = $rawBalances[chainIndex]

        if (!chainBalances || chainBalances.length === 0) {
          return {
            chainId: chain.chain_id,
            balances: []
          }
        }

        return {
          chainId: chain.chain_id,
          balances: chainBalances
        }
      })
    }
  ) as Readable<BalancesList>

  return derived([userAddress, balances], ([$userAddress, $balances]) => ({
    chains,
    userAddress: $userAddress,
    balances: $balances
  }))
}