import { derived, get, type Readable } from "svelte/store"
import type { IntentStore } from "./intents.ts"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { userAddrEvm } from "$lib/wallet/evm"
import { userAddressAptos } from "$lib/wallet/aptos"
import { userBalancesQuery } from "$lib/queries/balance"
import type { Chain, UserAddresses } from "$lib/types"
import { useQueryClient } from "@tanstack/svelte-query"
import type { Address } from "$lib/wallet/types"

export type AddressBalance = {
  balance: bigint
  gasToken: boolean
  address: Address
  symbol: string
  chain_id: string
}

export type NamedBalance = {
  balance: bigint
  address: string
  name: string | null
  symbol: string
  gasToken: boolean
  chain_id: string
}

export type EmptyBalance = {
  chain_id: string
}

export type Balance = AddressBalance | NamedBalance | EmptyBalance

export interface ContextStore {
  chains: Array<Chain>
  userAddress: UserAddresses
  sourceChain: Chain | undefined
  destinationChain: Chain | undefined
  balances: Array<Balance>
  assetInfo: Balance | undefined
}

export function createContextStore(intents: IntentStore): Readable<ContextStore> {
  const queryClient = useQueryClient()

  function queryData<T extends Array<unknown>>(
    key: Array<string>,
    filter?: (value: T[number]) => boolean
  ): T {
    const data = queryClient.getQueryData<T>(key) ?? []
    return (filter ? data.filter(filter) : data) as T
  }

  // Chain data
  const chains = queryData<Array<Chain>>(["chains"], chain => chain.enabled_staging)

  // User address data
  const userAddress = derived(
    [userAddrCosmos, userAddrEvm, userAddressAptos],
    ([cosmos, evm, aptos]) => ({ evm, aptos, cosmos })
  ) as Readable<UserAddresses>

  // Chain selections
  const sourceChain = derived(intents, intentsValue =>
    chains.find(chain => chain.chain_id === intentsValue.source)
  )

  const destinationChain = derived(intents, intentsValue =>
    chains.find(chain => chain.chain_id === intentsValue.destination)
  )

  const balances = derived(
    [
      intents,
      userAddress,
      userBalancesQuery({ chains, connected: true, userAddr: get(userAddress) })
    ],
    ([_intentsValue, _userAddressValue, rawBalances]) => {
      return rawBalances.flatMap((balanceResult, index) => {
        const chain = chains[index]
        if (!balanceResult?.isSuccess || balanceResult.data instanceof Error) {
          return []
        }
        return balanceResult.data.map(balance => ({
          ...balance,
          balance: BigInt(balance.balance),
          chain_id: chain.chain_id // Add chain_id to each balance
        }))
      })
    }
  )

  const assetInfo = derived([balances, intents], ([balancesValue, intentsValue]) =>
    balancesValue.find(x => x?.address === intentsValue.asset)
  )

  return derived(
    [userAddress, sourceChain, destinationChain, balances, assetInfo],
    ([$userAddress, $sourceChain, $destinationChain, $balances, $assetInfo]) => ({
      chains,
      userAddress: $userAddress,
      sourceChain: $sourceChain,
      destinationChain: $destinationChain,
      balances: $balances,
      assetInfo: $assetInfo
    })
  )
}
