import { derived, type Readable } from "svelte/store"
import type { Chain, ChainAsset } from "$lib/types"
import { useQueryClient } from "@tanstack/svelte-query"
import type { Address } from "$lib/wallet/types"
import { getSupportedAsset } from "$lib/utilities/helpers.ts"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context.ts"

export type BalanceRecord = {
  balance: bigint
  gasToken: boolean
  address: Address
  symbol: string
}

export interface SelectedAsset {
  address: string | undefined
  balance: bigint | undefined
  symbol: string | undefined
  decimals: number
  gasToken: boolean | undefined
  supported: ChainAsset | undefined
}

export interface IntentsStore {
  sourceChain: Chain | undefined
  destinationChain: Chain | undefined
  selectedAsset: SelectedAsset
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
    return $context.chains.find(chain => chain.chain_id === $intents.source)
  })

  const destinationChain = derived([rawIntents, context], ([$intents, $context]) =>
    $context.chains.find(chain => chain.chain_id === $intents.destination)
  )

  const asset = derived([context, rawIntents], ([$context, $intents]) =>
    $context.balances.find(x => x?.address === $intents.asset)
  )

  const supportedAsset = derived([sourceChain, asset], ([$sourceChain, $asset]) =>
    $sourceChain && $asset ? getSupportedAsset($sourceChain, $asset.address) : undefined
  )

  const selectedAsset = derived([asset, supportedAsset], ([$asset, $supportedAsset]) => ({
    address: $asset?.address,
    balance: $asset?.balance,
    symbol: getDisplaySymbol($asset, $supportedAsset),
    decimals: $supportedAsset?.decimals ?? 0,
    gasToken: $asset?.gasToken,
    supported: $supportedAsset,
  }))

  return derived(
    [sourceChain, destinationChain, selectedAsset, rawIntents],
    ([$sourceChain, $destinationChain, $selectedAsset, $rawIntents]) => ({
      sourceChain: $sourceChain,
      destinationChain: $destinationChain,
      selectedAsset: $selectedAsset,
      receiver: $rawIntents.receiver,
      amount: $rawIntents.amount
    })
  )
}
