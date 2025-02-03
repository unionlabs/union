import { derived, type Readable } from "svelte/store"
import type { Chain, ChainFeature, Ucs03Channel, UserAddresses } from "$lib/types"
import { userAddress } from "./balances.ts"
import type { BalanceData } from "$lib/queries/balance"
import { page } from "$app/stores"

export type ChainBalances = {
  chainId: string
  balances: Array<BalanceData>
}

export type BalancesList = Array<ChainBalances>

export interface ContextStore {
  chains: Array<Chain>
  userAddress: UserAddresses
  ucs03channels: Array<Ucs03Channel>
}

export function createContextStore(
  chains: Array<Chain>,
  ucs03channels: Array<Ucs03Channel>
): Readable<ContextStore> {
  return derived([userAddress, page], ([$userAddress, $page]) => {
    const enabledChains = chains.filter(chain => {
      const chainFeature = $page.data.features.find(
        (feature: ChainFeature) => feature.chain_id === chain.chain_id
      )
      return chainFeature?.features[0]?.transfer_submission
    })

    return {
      chains: enabledChains,
      userAddress: $userAddress,
      ucs03channels
    }
  })
}
