import { derived, type Readable } from "svelte/store"
import type { Chain, Ucs03Channel, UserAddresses } from "$lib/types"
import type { userBalancesQuery } from "$lib/queries/balance"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents"

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
  ucs03channels: Array<Ucs03Channel>
): Readable<ContextStore> {
  const baseTokenStore = derived([balancesQuery, rawIntents], ([$balances, $rawIntents]) => {
    const sourceChain = chains.find(c => c.chain_id === $rawIntents.source)
    if (!sourceChain) return []

    const chainBalances = $balances.find(b => b.data?.chain_id === $rawIntents.source)?.data
    return sourceChain.tokens.map(token => ({
      denom: token.denom,
      balance: chainBalances?.balances[token.denom] ?? "0"
    }))
  })

  return derived([userAddress, baseTokenStore], ([$userAddress, $baseTokens]) => {
    return {
      chains,
      baseTokens: $baseTokens,
      userAddress: $userAddress,
      ucs03channels
    }
  })
}
