import { Effect, Either, ParseResult, Schema } from "effect"
import {
  type AptosTransfer,
  type CosmosTransfer,
  type EVMTransfer,
  TransferSchema
} from "$lib/schema/transfer-args.ts"

export type ValidationSuccess = {
  isValid: true
  value: EVMTransfer | CosmosTransfer | AptosTransfer
  errors: []
  messages: []
  fieldErrors: Record<string, Array<string>> // empty if valid
}

export type ValidationFailure = {
  isValid: false
  value: undefined
  errors: unknown
  messages: Array<string>
  fieldErrors: Record<string, Array<string>>
}

export type ValidationResult = ValidationSuccess | ValidationFailure

const decodeAll = Schema.decodeUnknown(TransferSchema, { errors: "all" })

export function validateTransfer(args: unknown): ValidationResult {
  const decodeEither = Effect.runSync(Effect.either(decodeAll(args)))

  if (Either.isRight(decodeEither)) {
    return {
      isValid: true,
      value: decodeEither.right,
      errors: [],
      messages: [],
      fieldErrors: {}
    }
  }

  const parseError = decodeEither.left
  const arrayOutput = ParseResult.ArrayFormatter.formatErrorSync(parseError)

  const messages = arrayOutput.map(errObj => {
    return `Path: [${errObj.path.join(", ")}], message: ${errObj.message}`
  })

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

  return {
    isValid: false,
    value: undefined,
    errors: parseError,
    messages,
    fieldErrors
  }
}
