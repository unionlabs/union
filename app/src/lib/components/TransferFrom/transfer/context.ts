import { derived, type Readable } from "svelte/store"
import type { Chain, Ucs03Channel, UserAddresses } from "$lib/types"
import type { BalanceData, userBalancesQuery } from "$lib/queries/balance"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents"
import type { QueryObserverResult } from '@tanstack/query-core'

export interface TokenBalance {
  denom: string
  balance: string
}

export interface BalanceQueryResult {
  chain_id: string
  balances: Record<string, string>
}

export interface ContextStore {
  chains: Array<Chain>
  baseTokens: Array<TokenBalance>
  userAddress: UserAddresses
  ucs03channels: Array<Ucs03Channel>
}

export function createContextStore(
  rawIntents: RawIntentsStore,
  chains: Array<Chain>,
  userAddress: Readable<UserAddresses>,
  balancesQuery: ReturnType<typeof userBalancesQuery>,
  ucs03channels: Array<Ucs03Channel>,
): Readable<ContextStore> {
  // Create intermediate derived store for base tokens
  const baseTokenStore = derived(
    [balancesQuery, rawIntents],
    ([$balances, $rawIntents]) => {
      const sourceChain = chains.find(c => c.chain_id === $rawIntents.source)
      if (!sourceChain) return []

      const chainBalances = $balances.find(b => b.data?.chain_id === $rawIntents.source)?.data
      console.log('cb', chainBalances)
      return sourceChain.tokens.map(token => ({
        denom: token.denom,
        balance: chainBalances?.balances[token.denom] ?? "0"
      }))
    }
  )

  // Return the final derived store with flattened structure
  return derived(
    [userAddress, baseTokenStore],
    ([$userAddress, $baseTokens]) => {
      return {
        chains,
        baseTokens: $baseTokens,
        userAddress: $userAddress,
        ucs03channels
      }
    }
  )
}