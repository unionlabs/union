import type {Readable} from "svelte/store"
import {derived} from "svelte/store"
import type {IntentStore, FormFields, RawTransferIntents} from "./intents.ts"
import type {Chain} from "$lib/types"
import type {Balance, ContextStore} from "$lib/components/TransferFrom/transfer/context"
import {transferSchema} from "./schema.ts"
import {safeParse} from "valibot"

export type FieldErrors = Partial<Record<keyof FormFields, string>>

export interface ValidationStore extends Readable<FieldErrors> {
  validate: () => Promise<boolean>
}

interface ValidationContext {
  balances: Array<Balance>
  sourceChain: Chain | undefined
  destinationChain: Chain | undefined
  assetInfo: Balance | undefined
  chains: Array<Chain>
}

export function createValidationStore(
  intents: IntentStore,
  context: ContextStore
): ValidationStore {
  const errors = derived<
    [
      Readable<RawTransferIntents>,
      Readable<Array<Balance>>,
      Readable<Chain | undefined>,
      Readable<Chain | undefined>,
      Readable<Balance | undefined>
    ],
    FieldErrors
  >(
    [intents, context.balances, context.sourceChain, context.destinationChain, context.assetInfo],
    ([$intents, $balances, $sourceChain, $destinationChain, $assetInfo]) => {
      return validateAll({
        formFields: {
          source: $intents.source,
          destination: $intents.destination,
          asset: $intents.asset,
          receiver: $intents.receiver,
          amount: $intents.amount
        },
        balances: $balances,
        sourceChain: $sourceChain,
        destinationChain: $destinationChain,
        assetInfo: $assetInfo,
        chains: context.chains
      })
    }
  )

  function validateAll({
                         formFields,
                         balances,
                         sourceChain,
                         destinationChain,
                         assetInfo,
                         chains
                       }: {
    formFields: FormFields
    balances: Array<Balance>
    sourceChain: Chain | undefined
    destinationChain: Chain | undefined
    assetInfo: Balance | undefined
    chains: Array<Chain>
  }): FieldErrors {
    if (Object.values(formFields).every(value => !value)) {
      return {}
    }

    // First, try to parse with the schema including balance if available
    const parseInput = {
      ...formFields,
      balance: assetInfo && "balance" in assetInfo ? assetInfo.balance.toString() : undefined
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

    // Only proceed with business rules if schema validation passes
    return validateBusinessRules(formFields, {
      balances,
      sourceChain,
      destinationChain,
      assetInfo,
      chains
    })
  }

  function validateSchema(params: FormFields): FieldErrors {
    if (Object.values(params).every(value => !value)) {
      return {}
    }

    const result = safeParse(transferSchema, params)

    if (!result.success) {
      return result.issues.reduce((acc, issue) => {
        const fieldName = issue.path?.[0]?.key as keyof FormFields

        if (fieldName && !params[fieldName]) {
          return acc
        }

        if (fieldName) {
          acc[fieldName] = issue.message
        }
        return acc
      }, {} as FieldErrors)
    }

    return {}
  }

  function validateBusinessRules(formFields: FormFields, context: ValidationContext): FieldErrors {
    if (Object.values(formFields).every(value => !value)) {
      return {}
    }
    const errors: FieldErrors = {}
    
    if (formFields.source && formFields.destination && formFields.source === formFields.destination) {
      errors.destination = "Source and destination chains must be different"
    }

    return errors
  }

  return {
    subscribe: errors.subscribe,
    validate: () => {
      return new Promise(resolve => {
        let currentErrors: FieldErrors = {}
        const unsubscribe = errors.subscribe(value => {
          currentErrors = value
        })

        const isValid = Object.keys(currentErrors).length === 0
        unsubscribe()
        resolve(isValid)
      })
    }
  }
}
