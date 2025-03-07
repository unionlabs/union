import type { GetConnectorClientErrorType } from "@wagmi/core"
import { Data } from "effect"
import type {
  CreatePublicClientErrorType,
  CreateWalletClientErrorType,
  SwitchChainErrorType,
  WaitForTransactionReceiptErrorType,
  WriteContractErrorType
} from "viem"

export class CreateWalletClientError extends Data.TaggedError("CreateWalletClientError")<{
  cause: CreateWalletClientErrorType
}> {}

export class WaitForTransactionReceiptError extends Data.TaggedError(
  "WaitForTransactionReceiptError"
)<{
  cause: WaitForTransactionReceiptErrorType
}> {}

export class CreatePublicClientError extends Data.TaggedError("CreatePublicClientError")<{
  cause: CreatePublicClientErrorType
}> {}

export class WriteContractError extends Data.TaggedError("WriteContractError")<{
  cause: WriteContractErrorType
}> {}

export class SwitchChainError extends Data.TaggedError("SwitchChainError")<{
  cause: SwitchChainErrorType
}> {}

export class getAccountError extends Data.TaggedError("SwitchChainError")<{
  cause: string
}> {}

export class ConnectorClientError extends Data.TaggedError("ConnectorClientError")<{
  cause: GetConnectorClientErrorType
}> {}

export class AddressValidationError extends Data.TaggedError("AddressValidationError")<{
  input: string
  cause?: unknown | undefined
}> {}

export class AmountParsingError extends Data.TaggedError("AmountParsingError")<{
  input: string
  decimals: number
  cause?: unknown | undefined
}> {}

export class ChannelValidationError extends Data.TaggedError("ChannelValidationError")<{
  source_chain_id: string
  destination_chain_id: string
  cause?: unknown | undefined
}> {}

export class QuoteTokenError extends Data.TaggedError("QuoteTokenError")<{
  message: string
  cause?: unknown | undefined
}> {}

export type SubmitTransferError = WriteContractError | CreateWalletClientError