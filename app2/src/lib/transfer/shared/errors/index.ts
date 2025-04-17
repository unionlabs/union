import { Data } from "effect"

export class MissingTransferFieldsError extends Data.TaggedError("MissingTransferFieldsError")<{
  fields: Array<string>
}> {}

export class InsufficientFundsError extends Data.TaggedError("InsufficientFundsError")<{
  cause: string
}> {}

export class AllowanceCheckError extends Data.TaggedError("AllowanceCheckError")<{
  cause: unknown
}> {}

export class OrderCreationError extends Data.TaggedError("OrderCreationError")<{
  details: unknown
}> {}

export class BalanceLookupError extends Data.TaggedError("BalanceLookupError")<{
  chainId: string
  sender: string
  token: string
  cause: string
}> {}

export class CosmosQueryError extends Data.TaggedError("CosmosQueryError")<{
  token: string
  cause: unknown
}> {}

export type ContextFlowError =
  | MissingTransferFieldsError
  | InsufficientFundsError
  | OrderCreationError
  | BalanceLookupError
  | CosmosQueryError
  | AllowanceCheckError
