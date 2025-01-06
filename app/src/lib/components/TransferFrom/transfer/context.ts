import { derived, get, type Readable } from "svelte/store"
import type { RawIntentsStore } from "./raw-intents.ts"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { userAddrEvm } from "$lib/wallet/evm"
import { userAddressAptos } from "$lib/wallet/aptos"
import { userBalancesQuery } from "$lib/queries/balance"
import type { Chain, ChainAsset, UserAddresses } from "$lib/types"
import { useQueryClient } from "@tanstack/svelte-query"
import type { Address } from "$lib/wallet/types"
import { getSupportedAsset } from "$lib/utilities/helpers.ts"
import { showUnsupported } from "$lib/stores/user.ts"

export type BalanceRecord = {
  balance: bigint
  gasToken: boolean
  address: Address
  symbol: string
}

export type BalancesList = Array<BalanceRecord>

export interface AssetListItem {
  balance: BalanceRecord
  isSupported: boolean
  supportedAsset?: ChainAsset
  symbol: string
  sourceChain: Chain
}

export interface ContextStore {
  chains: Array<Chain>
  userAddress: UserAddresses
  balances: BalancesList
  assetsList: Array<AssetListItem>
}

const getDisplaySymbol = (
  balance: BalanceRecord | undefined,
  supportedAsset: ChainAsset | undefined
): string | undefined =>
  supportedAsset?.display_symbol || balance?.symbol || null || balance?.address || undefined

export function createContextStore(rawIntents: RawIntentsStore): Readable<ContextStore> {
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
    [
      rawIntents,
      userAddress,
      userBalancesQuery({ chains, connected: true, userAddr: get(userAddress) })
    ],
    ([$intents, _, $rawBalances]) => {
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

  const assetsList = derived([balances, rawIntents], ([$balances, $rawIntents]) => {
    const sourceChain = chains.find(chain => chain.chain_id === $rawIntents.source)
    if (!sourceChain) return []

    return $balances
      .map(balance => {
        const supportedAsset = getSupportedAsset(sourceChain, balance.address)
        const isSupported = Boolean(supportedAsset)

        if (!(get(showUnsupported) || isSupported)) return null

        return {
          balance,
          isSupported,
          supportedAsset,
          symbol: getDisplaySymbol(balance, supportedAsset) || balance.address,
          sourceChain
        }
      })
      .filter(Boolean) as Array<AssetListItem>
  })

  return derived([userAddress, balances, assetsList], ([$userAddress, $balances, $assetsList]) => ({
    chains,
    userAddress: $userAddress,
    balances: $balances,
    assetsList: $assetsList
  }))
}
