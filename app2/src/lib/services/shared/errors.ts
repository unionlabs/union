import { Data } from "effect"

export class AddressValidationError extends Data.TaggedError("AddressValidationError")<{
  input: string
  cause?: unknown | undefined
}> {}
