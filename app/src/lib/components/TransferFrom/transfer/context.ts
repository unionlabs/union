import { derived, type Readable } from "svelte/store"
import type { Chain, UserAddresses } from "$lib/types"
import type { Address } from "$lib/wallet/types"
import { balanceStore, userAddress } from "./balances"

export type BalanceRecord = {
  balance: bigint
  gasToken: boolean
  address: Address
  symbol: string
}

export type ChainBalances = {
  chainId: string
  balances: Array<BalanceRecord>
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
    ($rawBalances) => {
      if (!$rawBalances?.length) {
        return chains.map(chain => ({
          chainId: chain.chain_id,
          balances: []
        }))
      }

      return chains.map((chain, chainIndex) => {
        const balanceResult = $rawBalances[chainIndex]

        if (!balanceResult?.isSuccess || balanceResult.data instanceof Error) {
          console.log(`No balances fetched yet for chain ${chain.chain_id}`)
          return {
            chainId: chain.chain_id,
            balances: []
          }
        }

        return {
          chainId: chain.chain_id,
          balances: balanceResult.data.map(balance => ({
            ...balance,
            balance: BigInt(balance.balance),
            gasToken: balance.gasToken ?? false,
            address: balance.address as Address,
            symbol: balance.symbol || balance.address
          }))
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