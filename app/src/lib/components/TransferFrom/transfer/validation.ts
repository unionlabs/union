import type { Readable } from "svelte/store"
import { derived } from "svelte/store"
import type { IntentsStore, SelectedAsset } from "./intents.ts"
import type { Chain } from "$lib/types"
import type { BalanceRecord, ContextStore } from "$lib/components/TransferFrom/transfer/context"
import { transferSchema } from "./schema.ts"
import { safeParse } from "valibot"
import type {
  FormFields,
  RawIntentsStore
} from "$lib/components/TransferFrom/transfer/raw-intents.ts"

export type FieldErrors = Partial<Record<keyof FormFields, string>>

export interface ValidationStore {
  errors: FieldErrors
  isValid: boolean
}

export interface ValidationStoreAndMethods extends Readable<ValidationStore> {
  validate: () => Promise<boolean>
}

interface ValidationContext {
  balances: Array<BalanceRecord>
  sourceChain: Chain | undefined
  destinationChain: Chain | undefined
  selectedAsset: SelectedAsset
  chains: Array<Chain>
}

export function createValidationStore(
  rawIntents: RawIntentsStore,
  intents: Readable<IntentsStore>,
  context: Readable<ContextStore>
): ValidationStoreAndMethods {
  const store = derived([rawIntents, intents, context], ([$rawIntents, $intents, $context]) => {
    const formFields = {
      source: $rawIntents.source,
      destination: $rawIntents.destination,
      asset: $rawIntents.asset,
      receiver: $rawIntents.receiver,
      amount: $rawIntents.amount
    }

    // Check if all required fields have values
    const hasAllRequiredValues = Object.values(formFields).every(value => Boolean(value))

    // Parse input with schema if all fields are present
    let schemaValid = false
    if (hasAllRequiredValues) {
      const parseInput = {
        ...formFields,
        balance: $intents.selectedAsset.balance?.toString(),
        decimals: $intents.selectedAsset?.decimals
      }
      const schemaResult = safeParse(transferSchema, parseInput)
      schemaValid = schemaResult.success
    }

    // Always validate fields for error display
    const errors = validateAll({
      formFields,
      balances: $context.balances,
      sourceChain: $intents.sourceChain,
      destinationChain: $intents.destinationChain,
      selectedAsset: $intents.selectedAsset,
      chains: $context.chains
    })

    return {
      errors,
      // isValid only when all fields present, schema valid, and no validation errors
      isValid: hasAllRequiredValues && schemaValid && Object.keys(errors).length === 0
    }
  })

  function validateAll({
    formFields,
    balances,
    sourceChain,
    destinationChain,
    selectedAsset,
    chains
  }: {
    formFields: FormFields
    balances: Array<BalanceRecord>
    sourceChain: Chain | undefined
    destinationChain: Chain | undefined
    selectedAsset: SelectedAsset
    chains: Array<Chain>
  }): FieldErrors {
    const parseInput = {
      ...formFields,
      balance: selectedAsset.balance?.toString(),
      decimals: selectedAsset.supported?.decimals
    }

    const schemaResult = safeParse(transferSchema, parseInput)

    // If schema validation fails, return those errors
    if (!schemaResult.success) {
      return schemaResult.issues.reduce((acc, issue) => {
        const fieldName = issue.path?.[0]?.key as keyof FormFields
        if (fieldName && formFields[fieldName]) {
          // Only show error if field has a value
          acc[fieldName] = issue.message
        }
        return acc
      }, {} as FieldErrors)
    }

    // Only proceed with rules if schema validation passes
    return validateRules(formFields, {
      balances,
      sourceChain,
      destinationChain,
      selectedAsset,
      chains
    })
  }

  function validateRules(formFields: FormFields, _context: ValidationContext): FieldErrors {
    const errors: FieldErrors = {}

    if (
      formFields.source &&
      formFields.destination &&
      formFields.source === formFields.destination
    ) {
      errors.destination = "Source and destination chains must be different"
    }

    return errors
  }

  return {
    subscribe: store.subscribe,
    validate: () => {
      return new Promise(resolve => {
        let currentState: ValidationStore | undefined
        const unsubscribe = store.subscribe(value => {
          currentState = value
          unsubscribe()
          resolve(currentState?.isValid ?? false)
        })
      })
    }
  }
}
