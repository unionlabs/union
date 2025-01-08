import { derived, get, type Readable } from "svelte/store"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { userAddrEvm } from "$lib/wallet/evm"
import { userAddressAptos } from "$lib/wallet/aptos"
import { userBalancesQuery } from "$lib/queries/balance"
import type { Chain, UserAddresses } from "$lib/types"
import { useQueryClient } from "@tanstack/svelte-query"
import type { Address } from "$lib/wallet/types"

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

export function createContextStore(): Readable<ContextStore> {
  const queryClient = useQueryClient()

  const queryData = <T extends Array<unknown>>(
    key: Array<string>,
    filter?: (value: T[number]) => boolean
  ): T => {
    const data = queryClient.getQueryData<T>(key) ?? []
    return (filter ? data.filter(filter) : data) as T
  }

  const chains = queryData<Array<Chain>>(["chains"], chain => chain.enabled_staging)

  const userAddress = derived(
    [userAddrCosmos, userAddrEvm, userAddressAptos],
    ([cosmos, evm, aptos]) => ({ evm, aptos, cosmos })
  ) as Readable<UserAddresses>

  const balances = derived(
    [userAddress, userBalancesQuery({ chains, connected: true, userAddr: get(userAddress) })],
    ([_, $rawBalances]) => {
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
            balance: BigInt(balance.balance)
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
