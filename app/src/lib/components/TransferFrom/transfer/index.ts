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
import type {Chain, Ucs03Channel, UserAddresses} from "$lib/types"
import type { userBalancesQuery } from "$lib/queries/balance/index.ts"

export interface TransferStore {
  rawIntents: RawIntentsStore
  intents: Readable<IntentsStore>
  context: Readable<ContextStore>
  validation: Readable<ValidationStore>
}

export function createTransferStores(
  chains: Array<Chain>,
  userAddress :Readable<UserAddresses>,
  balances: ReturnType<typeof userBalancesQuery>,
  ucs03channels: Array<Ucs03Channel>
): TransferStore {
  const rawIntents = createRawIntentsStore()
  const context = createContextStore(rawIntents, chains, userAddress, balances, ucs03channels)
  const intents = createIntentStore(rawIntents, context)
  const validation = createValidationStore(rawIntents, intents, context)

  return {
    rawIntents,
    context,
    intents,
    validation
  }
}
