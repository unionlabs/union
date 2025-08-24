import type { Token, Ucs05 } from "@unionlabs/sdk"
import { Data } from "effect"

export class MissingTransferFieldsError extends Data.TaggedError("MissingTransferFieldsError")<{
  fields: Array<string>
}> {}

export class InsufficientFundsError extends Data.TaggedError("InsufficientFundsError")<{
  cause: string
}> {}

export class AllowanceCheckError extends Data.TaggedError("AllowanceCheckError")<{
  message: string
  cause?: unknown
}> {}

export class OrderCreationError extends Data.TaggedError("OrderCreationError")<{
  details: unknown
}> {}

export class BalanceLookupError extends Data.TaggedError("BalanceLookupError")<{
  chainId: string
  sender: Ucs05.AnyDisplay
  token: Token.Any
  cause: string
}> {}

export class CosmosQueryError extends Data.TaggedError("CosmosQueryError")<{
  token: string
  cause: unknown
}> {}

export class GenerateMultisigError extends Data.TaggedError("GenerateMultisigError")<{
  reason: string
  cause?: unknown
}> {}

export class GenericFlowError extends Data.TaggedError("GenericFlowError")<{
  message: string
  cause?: unknown
}> {}

export type ContextFlowError =
  | GenericFlowError
  | MissingTransferFieldsError
  | InsufficientFundsError
  | OrderCreationError
  | BalanceLookupError
  | CosmosQueryError
  | AllowanceCheckError
  | GenerateMultisigError
