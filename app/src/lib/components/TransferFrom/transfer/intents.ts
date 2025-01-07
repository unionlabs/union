import { derived, type Readable } from "svelte/store"
import type { Chain, ChainAsset } from "$lib/types"
import { useQueryClient } from "@tanstack/svelte-query"
import { getSupportedAsset } from "$lib/utilities/helpers.ts"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import type { ContextStore, BalanceRecord } from "$lib/components/TransferFrom/transfer/context.ts"
import { showUnsupported } from "$lib/stores/user.ts"
import { get } from "svelte/store"

export type AssetListItem = {
  balance: BalanceRecord
  isSupported: boolean
  supportedAsset?: ChainAsset
  symbol: string
  sourceChain: Chain
}

export interface SelectedAsset {
  address: string | null
  balance: bigint | null
  symbol: string | null
  decimals: number
  gasToken: boolean
  supported: ChainAsset | null
}

export interface IntentsStore {
  sourceChain: Chain | null
  destinationChain: Chain | null
  selectedAsset: SelectedAsset
  sourceAssets: Array<AssetListItem>
  receiver: string
  amount: string
}

const getDisplaySymbol = (
  balance: BalanceRecord | undefined,
  supportedAsset: ChainAsset | undefined
): string | undefined =>
  supportedAsset?.display_symbol || balance?.symbol || balance?.address || undefined

export function createIntentStore(
  rawIntents: RawIntentsStore,
  context: Readable<ContextStore>
): Readable<IntentsStore> {
  const queryClient = useQueryClient()

  const sourceChain = derived([rawIntents, context], ([$intents, $context]) => {
    return $context.chains.find(chain => chain.chain_id === $intents.source) ?? null
  })

  const destinationChain = derived(
    [rawIntents, context],
    ([$intents, $context]) =>
      $context.chains.find(chain => chain.chain_id === $intents.destination) ?? null
  )

  //Assets of selected chain
  const sourceAssets = derived([context, sourceChain], ([$context, $sourceChain]) => {
    if (!$sourceChain) return []

    const chainBalances =
      $context.balances.find(chain => chain.chainId === $sourceChain.chain_id)?.balances || []

    return chainBalances
      .map(balance => {
        const supportedAsset = getSupportedAsset($sourceChain, balance.address)
        const isSupported = Boolean(supportedAsset)

        if (!(get(showUnsupported) || isSupported)) return null

        return {
          balance,
          isSupported,
          supportedAsset,
          symbol: getDisplaySymbol(balance, supportedAsset) || balance.address,
          sourceChain: $sourceChain
        }
      })
      .filter(Boolean) as Array<AssetListItem>
  })

  // Find the specific asset in the source chain assets
  const asset = derived(
    [sourceAssets, rawIntents],
    ([$assets, $intents]) => $assets.find(x => x.balance.address === $intents.asset)?.balance
  )

  //Get supported asset info (if supported)
  const supportedAsset = derived([sourceChain, asset], ([$sourceChain, $asset]) =>
    $sourceChain && $asset ? getSupportedAsset($sourceChain, $asset.address) : undefined
  )

  //Create th selected asset with all info
  const selectedAsset = derived([asset, supportedAsset], ([$asset, $supportedAsset]) => ({
    address: $asset?.address ?? "",
    balance: $asset?.balance ?? 0n,
    symbol: getDisplaySymbol($asset, $supportedAsset) ?? "",
    decimals: $supportedAsset?.decimals ?? 0,
    gasToken: $asset?.gasToken ?? false,
    supported: $supportedAsset ?? null
  }))

  return derived(
    [sourceChain, destinationChain, selectedAsset, sourceAssets, rawIntents],
    ([$sourceChain, $destinationChain, $selectedAsset, $sourceAssets, $rawIntents]) => ({
      sourceChain: $sourceChain,
      destinationChain: $destinationChain,
      selectedAsset: $selectedAsset,
      sourceAssets: $sourceAssets,
      receiver: $rawIntents.receiver,
      amount: $rawIntents.amount
    })
  )
}
