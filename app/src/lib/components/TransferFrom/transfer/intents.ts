import { derived, type Readable } from "svelte/store"
import type { Chain } from "$lib/types"
import type { RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context.ts"
import type { BalanceData } from "$lib/queries/balance"

export type AssetListItem = BalanceData & {
  sourceChain: Chain
}

export interface IntentsStore {
  chains: Array<Chain>
  sourceChain: Chain | null
  destinationChain: Chain | null
  baseTokens: Array<{ denom: string; balance: string }>
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

  const baseTokens = derived([context, sourceChain], ([$context, $sourceChain]) => {
    if (!$sourceChain) return []
    let balances = $context.balances.find(c => c.data?.chain_id === $sourceChain.chain_id)
    let baseTokens = $sourceChain.tokens.map(token => ({
      denom: token.denom,
      balance: balances?.data?.balances[token.denom] ?? "0"
    }))

    return baseTokens
  })

  return derived(
    [sourceChain, destinationChain, baseTokens, rawIntents, context],
    ([$sourceChain, $destinationChain, $baseTokens, $rawIntents, $context]) => ({
      chains: $context.chains,
      sourceChain: $sourceChain,
      destinationChain: $destinationChain,
      baseTokens: $baseTokens,
      receiver: $rawIntents.receiver,
      amount: $rawIntents.amount
    })
  )
}
