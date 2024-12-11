import type { Readable } from "svelte/store"
import { derived } from "svelte/store"
import type {IntentStore, FormFields, RawTransferIntents} from "./intents"
import type { Chain, UserAddresses } from "$lib/types"
import type { Balance, ContextStore } from "$lib/components/TransferFrom/transfer/context"
import { transferSchema } from "./schema.ts"
import { safeParse } from "valibot"

export type FieldErrors = Partial<Record<keyof FormFields, string>>

export interface ValidationStore extends Readable<FieldErrors> {
  validate: () => Promise<boolean>
}

interface ValidationContext {
  balances: Balance[]
  sourceChain: Chain | undefined
  destinationChain: Chain | undefined
  assetInfo: Balance | undefined
  chains: Chain[]
}

export function createValidationStore(
  intents: IntentStore,
  context: ContextStore
): ValidationStore {
  const errors = derived<
    [
      Readable<RawTransferIntents>,
      Readable<Balance[]>,
      Readable<Chain | undefined>,
      Readable<Chain | undefined>,
      Readable<Balance | undefined>
    ],
    FieldErrors
  >(
    [
      intents,
      context.balances,
      context.sourceChain,
      context.destinationChain,
      context.assetInfo
    ],
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
    balances: Balance[]
    sourceChain: Chain | undefined
    destinationChain: Chain | undefined
    assetInfo: Balance | undefined
    chains: Chain[]
  }): FieldErrors {
    const schemaErrors = validateSchema(formFields)
    const businessErrors = validateBusinessRules(formFields, {
      balances,
      sourceChain,
      destinationChain,
      assetInfo,
      chains
    })

    return {
      ...schemaErrors,
      ...businessErrors
    }
  }

  function validateSchema(params: FormFields): FieldErrors {
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

  function validateBusinessRules(
    formFields: FormFields,
    context: ValidationContext
  ): FieldErrors {
    const errors: FieldErrors = {}

    // Validate chains
    if (formFields.source === formFields.destination) {
      errors.destination = "Source and destination chains must be different"
    }

    // Validate chain existence
    if (!context.sourceChain) {
      errors.source = "Invalid source chain"
    }
    if (!context.destinationChain) {
      errors.destination = "Invalid destination chain"
    }

    // Validate amount against balance
    if (formFields.amount && context.assetInfo && 'balance' in context.assetInfo) {
      const amount = parseFloat(formFields.amount)
      const balance = Number(context.assetInfo.balance)
      if (amount > balance) {
        errors.amount = "Insufficient balance"
      }
    }

    // Add any other cross-field or context-dependent validations

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