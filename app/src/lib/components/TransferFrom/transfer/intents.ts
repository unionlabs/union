import { derived, type Readable } from "svelte/store"
import type { Chain } from "$lib/types"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context.ts"
import { showUnsupported } from "$lib/stores/user.ts"
import { get } from "svelte/store"
import type { BalanceData } from "$lib/queries/balance"

export type AssetListItem = BalanceData & {
  sourceChain: Chain
}

export type SelectedAsset = BalanceData | null

export interface IntentsStore {
  sourceChain: Chain | null
  destinationChain: Chain | null
  selectedAsset: SelectedAsset
  sourceAssets: Array<AssetListItem>
  receiver: string
  amount: string
}

export function createIntentStore(
  rawIntents: RawIntentsStore,
  context: Readable<ContextStore>
): Readable<IntentsStore> {
  const sourceChain = derived([rawIntents, context], ([$intents, $context]) => {
    return $context.chains.find(chain => chain.chain_id === $intents.source) ?? null
  })

  const destinationChain = derived(
    [rawIntents, context],
    ([$intents, $context]) =>
      $context.chains.find(chain => chain.chain_id === $intents.destination) ?? null
  )

  const sourceAssets = derived([context, sourceChain], ([$context, $sourceChain]) => {
    if (!$sourceChain) return []

    const chainBalances =
      $context.balances.find(chain => chain.chainId === $sourceChain.chain_id)?.balances || []

    return chainBalances
      .filter(balance => get(showUnsupported) || balance.metadata.metadata_level !== "none")
      .map(balance => ({
        ...balance,
        sourceChain: $sourceChain
      }))
  })

  const selectedAsset = derived([sourceAssets, rawIntents], ([$sourceAssets, $rawIntents]) => {
    return $sourceAssets.find(x => x.metadata.denom === $rawIntents.asset) ?? null
  })

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
