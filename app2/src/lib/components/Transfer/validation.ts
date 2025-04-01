import { Data, Effect, Either, ParseResult, Schema } from "effect"
import {
  type AptosTransfer,
  type CosmosTransfer,
  type EVMTransfer, TransferSchema,
} from "@unionlabs/sdk/schema"
import type { TransferSubmission as EvmTransferSubmission } from "$lib/services/transfer-ucs03-evm"
import type { TransferSubmission as CosmosTransferSubmission } from "$lib/services/transfer-ucs03-cosmos"
import type { TransferSubmission as AptosTransferSubmission } from "$lib/services/transfer-ucs03-aptos"

export type TransferState = Data.TaggedEnum<{
  Empty: {}
  Evm: { state: EvmTransferSubmission }
  Cosmos: { state: CosmosTransferSubmission }
  Aptos: { state: AptosTransferSubmission }
}>

const TransferStateInternal = Data.taggedEnum<TransferState>()

export const TransferState = {
  Empty: () => TransferStateInternal.Empty(),
  Evm: (state: EvmTransferSubmission) => TransferStateInternal.Evm({ state }),
  Aptos: (state: AptosTransferSubmission) => TransferStateInternal.Aptos({ state }),
  Cosmos: (state: CosmosTransferSubmission) => TransferStateInternal.Cosmos({ state })
}

export type TransferStateUnion = TransferState

export type ValidationResult = Data.TaggedEnum<{
  Success: {
    value: EVMTransfer | CosmosTransfer | AptosTransfer
    fieldErrors: Record<string, never>
  }
  Failure: {
    errors: unknown
    messages: Array<string>
    fieldErrors: Record<string, Array<string>>
  }
}>

const ValidationResultInternal = Data.taggedEnum<ValidationResult>()

export const isValid = (result: ValidationResult): boolean => result._tag === "Success"

export function validateTransfer(args: unknown): ValidationResult {
  const decodeEither = Effect.runSync(Effect.either(decodeAll(args)))

  if (Either.isRight(decodeEither)) {
    return ValidationResultInternal.Success({
      value: decodeEither.right,
      fieldErrors: {}
    })
  }

  const parseError = decodeEither.left
  const arrayOutput = ParseResult.ArrayFormatter.formatErrorSync(parseError)

  const messages = arrayOutput.map(
    errObj => `Path: [${errObj.path.join(", ")}], message: ${errObj.message}`
  )

  const fieldErrors: Record<string, Array<string>> = {}

  for (const { path, message } of arrayOutput) {
    const [field] = path

    if (typeof field === "string") {
      if (!fieldErrors[field]) {
        fieldErrors[field] = []
      }
      fieldErrors[field].push(message)
    } else {
      if (!fieldErrors["_general"]) {
        fieldErrors["_general"] = []
      }
      fieldErrors["_general"].push(message)
    }
  }

  return ValidationResultInternal.Failure({
    errors: parseError,
    messages,
    fieldErrors
  })
}

const decodeAll = Schema.decodeUnknown(TransferSchema, { errors: "all" })
