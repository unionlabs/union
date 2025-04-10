import { Data } from "effect"

export class MissingTransferFieldsError extends Data.TaggedError("MissingTransferFieldsError")<{
  fields: string[]
}> {}

export class InsufficientFundsError extends Data.TaggedError("InsufficientFundsError")<{
  reason: string
}> {}

export class AllowanceCheckError extends Data.TaggedError("AllowanceCheckError")<{
  details: unknown
}> {}

export class OrderCreationError extends Data.TaggedError("OrderCreationError")<{
  details: unknown
}> {}

export class BalanceLookupError extends Data.TaggedError("BalanceLookupError")<{
  chainId: string
  sender: string
  token: string
  reason: string
}> {}

export class CosmosQueryError extends Data.TaggedError("CosmosQueryError")<{
  token: string
  details: unknown
}> {}

export class EvmAllowanceCheckError extends Data.TaggedError("EvmAllowanceCheckError")<{
  details: unknown
}> {}

export type TransferFlowError =
  | MissingTransferFieldsError
  | InsufficientFundsError
  | AllowanceCheckError
  | OrderCreationError
  | BalanceLookupError
  | CosmosQueryError
  | EvmAllowanceCheckError

