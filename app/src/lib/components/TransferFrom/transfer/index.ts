import { createIntentStore, type IntentStore } from "./intents.ts"
import type { Readable } from "svelte/store"
import type { Chain, UserAddresses } from "$lib/types.ts"
import { type Balance, createContextStore } from "$lib/components/TransferFrom/transfer/context.ts"
import {
  createValidationStore,
  type ValidationStore
} from "$lib/components/TransferFrom/transfer/validation.ts"

export interface TransferStore {
  intents: IntentStore
  context: {
    chains: Array<Chain>
    userAddress: Readable<UserAddresses>
    sourceChain: Readable<Chain | undefined>
    destinationChain: Readable<Chain | undefined>
    balances: Readable<Array<Balance>>
    assetInfo: Readable<Balance | undefined>
  }
  validation: ValidationStore
}

export function createTransferStore(): TransferStore {
  const intents = createIntentStore()
  const context = createContextStore(intents)
  const validation = createValidationStore(intents, context)

  return {
    intents,
    context,
    validation
  }
}
