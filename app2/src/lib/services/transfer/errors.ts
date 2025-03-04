import type { GetConnectorClientErrorType } from "@wagmi/core"
import { Data } from "effect"
import type {
  CreatePublicClientErrorType,
  CreateWalletClientErrorType,
  SendTransactionErrorType,
  SwitchChainErrorType,
  WaitForTransactionReceiptErrorType
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

export class SendTransactionError extends Data.TaggedError("SendTransactionError")<{
  cause: SendTransactionErrorType
}> {}

export class SwitchChainError extends Data.TaggedError("SwitchChainError")<{
  cause: SwitchChainErrorType
}> {}

export class ConnectorClientError extends Data.TaggedError("ConnectorClientError")<{
  cause: GetConnectorClientErrorType
}> {}

export type SubmitTransferError = SendTransactionError | CreateWalletClientError
