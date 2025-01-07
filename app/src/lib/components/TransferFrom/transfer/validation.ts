import type { Readable } from "svelte/store"
import { derived } from "svelte/store"
import type { IntentsStore } from "./intents.ts"
import type { Chain, ChainAsset } from "$lib/types"
import type { ContextStore } from "$lib/components/TransferFrom/transfer/context"
import { transferSchema } from "./schema.ts"
import { safeParse } from "valibot"
import type { FormFields, RawIntentsStore } from "$lib/components/TransferFrom/transfer/raw-intents"

export type FieldErrors = Partial<Record<keyof FormFields, string>>

export interface ValidTransfer {
  sourceChain: Chain
  destinationChain: Chain
  asset: {
    address: string
    balance: bigint
    symbol: string
    decimals: number
    gasToken: boolean
    supported: ChainAsset
  }
  receiver: string
  amount: string
}

export interface InvalidValidationStore {
  transfer: undefined
  errors: FieldErrors
  isValid: false
}

export interface ValidValidationStore {
  transfer: ValidTransfer
  errors: FieldErrors
  isValid: true
}

export type ValidationStore = InvalidValidationStore | ValidValidationStore

export function createValidationStore(
  rawIntents: RawIntentsStore,
  intents: Readable<IntentsStore>,
  context: Readable<ContextStore>
): Readable<ValidationStore> {
  const errors = derived([rawIntents, intents, context], ([$rawIntents, $intents, _$context]) => {
    const formFields = {
      source: $rawIntents.source,
      destination: $rawIntents.destination,
      asset: $rawIntents.asset,
      receiver: $rawIntents.receiver,
      amount: $rawIntents.amount
    }

    if (formFields.asset) {
      const errors: FieldErrors = {}
      if (!formFields.amount) errors.amount = "Amount is required"
      if (!formFields.receiver) errors.receiver = "Receiver is required"
      if (Object.keys(errors).length > 0) return errors
    }

    const parseInput = {
      ...formFields,
      balance: $intents.selectedAsset.balance ?? 0n,
      decimals: $intents.selectedAsset.decimals ?? 0
    }

    const schemaResult = safeParse(transferSchema, parseInput)
    if (!schemaResult.success) {
      return schemaResult.issues.reduce((acc, issue) => {
        const fieldName = issue.path?.[0]?.key as keyof FormFields
        if (fieldName && formFields[fieldName]) {
          acc[fieldName] = issue.message
        }
        return acc
      }, {} as FieldErrors)
    }

    if (
      formFields.source &&
      formFields.destination &&
      formFields.source === formFields.destination
    ) {
      return { destination: "Source and destination chains must be different" }
    }

    return {}
  })

  const transfer = derived([rawIntents, intents, errors], ([$rawIntents, $intents, $errors]) => {
    if (Object.keys($errors).length > 0 || !$rawIntents.amount || !$rawIntents.receiver)
      return undefined

    if (
      !(
        $intents.sourceChain &&
        $intents.destinationChain &&
        $intents.selectedAsset.address &&
        $intents.selectedAsset.balance &&
        $intents.selectedAsset.symbol &&
        $intents.selectedAsset.supported
      )
    ) {
      return undefined
    }

    return {
      sourceChain: $intents.sourceChain,
      destinationChain: $intents.destinationChain,
      asset: {
        address: $intents.selectedAsset.address,
        balance: $intents.selectedAsset.balance,
        symbol: $intents.selectedAsset.symbol,
        decimals: $intents.selectedAsset.decimals,
        gasToken: $intents.selectedAsset.gasToken,
        supported: $intents.selectedAsset.supported
      },
      receiver: $rawIntents.receiver,
      amount: $rawIntents.amount
    } as ValidTransfer
  })

  return derived([transfer, errors], ([$transfer, $errors]): ValidationStore => {
    const isValid = $transfer !== undefined

    return isValid
      ? { transfer: $transfer as ValidTransfer, errors: $errors, isValid: true }
      : { transfer: undefined, errors: $errors, isValid: false }
  })
}
