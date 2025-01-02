import { derived, get, type Readable } from "svelte/store"
import type { IntentStore } from "./intents.ts"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { userAddrEvm } from "$lib/wallet/evm"
import { userAddressAptos } from "$lib/wallet/aptos"
import { userBalancesQuery } from "$lib/queries/balance"
import type { Chain, ChainAsset, UserAddresses } from "$lib/types"
import { useQueryClient } from "@tanstack/svelte-query"
import type { Address } from "$lib/wallet/types"
import { getSupportedAsset } from "$lib/utilities/helpers.ts"

export type BalanceRecord = {
  balance: bigint
  gasToken: boolean
  address: Address
  symbol: string
}

export type BalancesList = Array<BalanceRecord>

export interface ContextStore {
  chains: Array<Chain>
  userAddress: UserAddresses
  sourceChain: Chain
  destinationChain: Chain | undefined
  balances: BalancesList
  assetBalance: BalanceRecord | undefined
  assetInfo: ChainAsset | undefined
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

  const chains = queryData<Array<Chain>>(["chains"], chain => chain.enabled_staging)

  const userAddress = derived(
    [userAddrCosmos, userAddrEvm, userAddressAptos],
    ([cosmos, evm, aptos]) => ({ evm, aptos, cosmos })
  ) as Readable<UserAddresses>

  const sourceChain = derived(intents, intentsValue => {
    const chain = chains.find(chain => chain.chain_id === intentsValue.source)
    if (!chain) {
      throw new Error(`No chain found for source ${intentsValue.source}`)
    }
    return chain
  })

  const destinationChain = derived(intents, intentsValue =>
    chains.find(chain => chain.chain_id === intentsValue.destination)
  )

  const balances = derived(
    [
      intents,
      userAddress,
      userBalancesQuery({ chains, connected: true, userAddr: get(userAddress) })
    ],
    ([$intents, _userAddressValue, $rawBalances]) => {
      const sourceChain = chains.find(chain => chain.chain_id === $intents.source)
      if (!sourceChain) return []

      const chainIndex = chains.findIndex(c => c.chain_id === sourceChain.chain_id)
      const balanceResult = $rawBalances[chainIndex]

      if (!balanceResult?.isSuccess || balanceResult.data instanceof Error) {
        console.log("No balances fetched yet for selected chain")
        return []
      }

      return balanceResult.data.map(balance => ({
        ...balance,
        balance: BigInt(balance.balance)
      }))
    }
  ) as Readable<BalancesList>

  const assetBalance = derived([balances, intents], ([$balances, $intents]) =>
    $balances.find(x => x?.address === $intents.asset)
  )

  const assetInfo = derived([sourceChain, intents], ([$sourceChain, $intents]) => {
    if ($intents.asset) {
      return getSupportedAsset($sourceChain, $intents.asset)
    }
    return undefined
  })

  return derived(
    [userAddress, sourceChain, destinationChain, balances, assetBalance, assetInfo],
    ([$userAddress, $sourceChain, $destinationChain, $balances, $assetBalance, $assetInfo]) => ({
      chains,
      userAddress: $userAddress,
      sourceChain: $sourceChain,
      destinationChain: $destinationChain,
      balances: $balances,
      assetBalance: $assetBalance,
      assetInfo: $assetInfo
    })
  )
}
