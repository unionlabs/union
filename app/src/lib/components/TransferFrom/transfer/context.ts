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
  symbol: string // Pre-computed symbol
}

export interface SelectedAsset {
  address: Address | undefined
  balance: bigint | undefined
  symbol: string | undefined
  decimals: number | undefined
  gasToken: boolean | undefined
  supported: ChainAsset | undefined
  raw: BalanceRecord | undefined
}

export interface ContextStore {
  chains: Array<Chain>
  sourceChain: Chain
  destinationChain: Chain | undefined
  userAddress: UserAddresses
  balances: BalancesList
  assetsList: Array<AssetListItem>
  selectedAsset: SelectedAsset
}

const getDisplaySymbol = (
  balance: BalanceRecord | undefined,
  supportedAsset: ChainAsset | undefined
): string | undefined =>
  supportedAsset?.display_symbol || balance?.symbol || null || balance?.address || undefined

export function createContextStore(intents: IntentStore): Readable<ContextStore> {
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

  const sourceChain = derived(intents, $intents => {
    const chain = chains.find(chain => chain.chain_id === $intents.source)
    if (!chain) throw new Error(`No chain found for source ${$intents.source}`)
    return chain
  })

  const destinationChain = derived(intents, $intents =>
    chains.find(chain => chain.chain_id === $intents.destination)
  )

  const balances = derived(
    [
      intents,
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

  const assetsList = derived(
    [balances, sourceChain, showUnsupported],
    ([$balances, $sourceChain, $showUnsupported]) =>
      $balances
        .map(balance => {
          const supportedAsset = getSupportedAsset($sourceChain, balance.address)
          const isSupported = Boolean(supportedAsset)

          if (!($showUnsupported || isSupported)) return null

          return {
            balance,
            isSupported,
            supportedAsset,
            symbol: getDisplaySymbol(balance, supportedAsset) || balance.address
          }
        })
        .filter(Boolean) as Array<AssetListItem>
  )

  const asset = derived([balances, intents], ([$balances, $intents]) =>
    $balances.find(x => x?.address === $intents.asset)
  )

  const supportedAsset = derived(
    [sourceChain, asset],
    ([$sourceChain, $asset]) => $asset && getSupportedAsset($sourceChain, $asset.address)
  )

  const selectedAsset = derived([asset, supportedAsset], ([$asset, $supportedAsset]) => ({
    address: $asset?.address,
    balance: $asset?.balance,
    symbol: getDisplaySymbol($asset, $supportedAsset),
    decimals: $supportedAsset?.decimals ?? 0,
    gasToken: $asset?.gasToken,
    supported: $supportedAsset,
    raw: $asset
  }))

  return derived(
    [userAddress, sourceChain, destinationChain, balances, assetsList, selectedAsset],
    ([$userAddress, $sourceChain, $destinationChain, $balances, $assetsList, $selectedAsset]) => ({
      chains,
      userAddress: $userAddress,
      sourceChain: $sourceChain,
      destinationChain: $destinationChain,
      balances: $balances,
      assetsList: $assetsList,
      selectedAsset: $selectedAsset
    })
  )
}
