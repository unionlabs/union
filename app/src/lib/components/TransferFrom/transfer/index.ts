import type { Readable } from "svelte/store"
import { createIntentStore, type IntentsStore } from "./intents.ts"
import {
  type ContextStore,
  createContextStore
} from "$lib/components/TransferFrom/transfer/context.ts"
import {
  createRawIntentsStore,
  type RawIntentsStore
} from "$lib/components/TransferFrom/transfer/raw-intents.ts"
import {
  createValidationStore,
  type ValidationStore
} from "$lib/components/TransferFrom/transfer/validation.ts"
import type { Chain, Ucs03Channel } from "$lib/types"
import type { userBalancesQuery } from "$lib/queries/balance/index.ts"

export interface TransferStore {
  rawIntents: RawIntentsStore
  intents: Readable<IntentsStore>
  context: Readable<ContextStore>
  validation: Readable<ValidationStore>
}

export function createTransferStore(
  chains: Array<Chain>,
  balances: ReturnType<typeof userBalancesQuery>,
  ucs03channels: Array<Ucs03Channel>
): TransferStore {
  const rawIntents = createRawIntentsStore()
  const context = createContextStore(chains, ucs03channels)
  const intents = createIntentStore(rawIntents, context, balances)
  const validation = createValidationStore(rawIntents, intents, context)

  return {
    rawIntents,
    context,
    intents,
    validation
  }
}
