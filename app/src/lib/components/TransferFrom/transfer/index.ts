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
import type { Chain, UserAddresses } from "$lib/types"
import type { QueryObserverResult } from "@tanstack/svelte-query"


export interface TransferStore {
  rawIntents: RawIntentsStore
  intents: Readable<IntentsStore>
  context: Readable<ContextStore>
  validation: Readable<ValidationStore>
}

export function createTransferStore(chains: Array<Chain>): TransferStore {
  const rawIntents = createRawIntentsStore()
  const context = createContextStore(chains)
  const intents = createIntentStore(rawIntents, context)
  const validation = createValidationStore(rawIntents, intents, context)

  return {
    rawIntents,
    context,
    intents,
    validation
  }
}