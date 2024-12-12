import { type Readable } from "svelte/store"
import { createIntentStore, type IntentStore } from "./intents.ts"
import { type ContextStore, createContextStore} from "$lib/components/TransferFrom/transfer/context.ts"
import {
  createValidationStore, type ValidationStoreAndMethods
} from "$lib/components/TransferFrom/transfer/validation.ts"

export interface TransferStore {
  intents: IntentStore
  context: Readable<ContextStore>
  validation: ValidationStoreAndMethods
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