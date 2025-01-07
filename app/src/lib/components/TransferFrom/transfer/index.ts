import type { Readable } from "svelte/store"
import { createIntentStore, type IntentsStore } from "./intents.ts"
import {
  type ContextStore,
  createContextStore
} from "$lib/components/TransferFrom/transfer/context.ts"
import {
  createValidationStore,
  type ValidationStoreAndMethods
} from "$lib/components/TransferFrom/transfer/validation.ts"
import {
  createRawIntentsStore,
  type RawIntentsStore
} from "$lib/components/TransferFrom/transfer/raw-intents.ts"

export interface TransferStore {
  rawIntents: RawIntentsStore
  intents: Readable<IntentsStore>
  context: Readable<ContextStore>
  validation: ValidationStoreAndMethods
}

export function createTransferStore(): TransferStore {
  const rawIntents = createRawIntentsStore()
  const context = createContextStore()
  const intents = createIntentStore(rawIntents, context)
  const validation = createValidationStore(rawIntents, intents, context)

  return {
    rawIntents,
    context,
    intents,
    validation
  }
}
