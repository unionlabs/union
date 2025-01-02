import type { Readable } from "svelte/store"
import { derived } from "svelte/store"
import type { IntentStore, FormFields } from "./intents.ts"
import type { Chain, ChainAsset } from "$lib/types"
import type { BalanceRecord, ContextStore } from "$lib/components/TransferFrom/transfer/context"
import { transferSchema } from "./schema.ts"
import { safeParse } from "valibot"

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
  assetBalance: BalanceRecord | undefined
  assetInfo: ChainAsset | undefined
  chains: Array<Chain>
}

export function createValidationStore(
  intents: IntentStore,
  context: Readable<ContextStore>
): ValidationStoreAndMethods {
  const store = derived([intents, context], ([$intents, $context]) => {
    const errors = validateAll({
      formFields: {
        source: $intents.source,
        destination: $intents.destination,
        asset: $intents.asset,
        receiver: $intents.receiver,
        amount: $intents.amount
      },
      balances: $context.balances,
      sourceChain: $context.sourceChain,
      destinationChain: $context.destinationChain,
      assetBalance: $context.assetBalance,
      assetInfo: $context.assetInfo,
      chains: $context.chains
    })

    return {
      errors,
      isValid: Object.keys(errors).length === 0
    }
  })

  function validateAll({
    formFields,
    balances,
    sourceChain,
    destinationChain,
    assetBalance,
    assetInfo,
    chains
  }: {
    formFields: FormFields
    balances: Array<BalanceRecord>
    sourceChain: Chain
    destinationChain: Chain | undefined
    assetBalance: BalanceRecord | undefined
    assetInfo: ChainAsset | undefined
    chains: Array<Chain>
  }): FieldErrors {
    if (Object.values(formFields).every(value => !value)) {
      return {}
    }

    const parseInput = {
      ...formFields,
      balance: assetBalance?.balance.toString(),
      decimals: assetInfo?.decimals
    }

    const schemaResult = safeParse(transferSchema, parseInput)

    // If schema validation fails, return those errors immediately
    if (!schemaResult.success) {
      return schemaResult.issues.reduce((acc, issue) => {
        const fieldName = issue.path?.[0]?.key as keyof FormFields
        if (fieldName && !formFields[fieldName]) {
          return acc
        }
        if (fieldName) {
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
      assetBalance,
      assetInfo,
      chains
    })
  }

  function validateRules(formFields: FormFields, _context: ValidationContext): FieldErrors {
    if (Object.values(formFields).every(value => !value)) {
      return {}
    }
    const errors: FieldErrors = {}

    //Example of a rule
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
