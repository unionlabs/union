import type { GetConnectorClientErrorType } from "@wagmi/core"
import { Data } from "effect"
import type {
  CreatePublicClientErrorType,
  CreateWalletClientErrorType,
  SendTransactionErrorType,
  SwitchChainErrorType,
  WaitForTransactionReceiptErrorType,
} from "viem"
import { CosmosSwitchChainError } from "../transfer-ucs03-cosmos"
import { EvmSwitchChainError } from "../transfer-ucs03-evm"

export class CreateWalletClientError extends Data.TaggedError("CreateWalletClientError")<{
  cause: CreateWalletClientErrorType
}> {}

export class WaitForTransactionReceiptError extends Data.TaggedError(
  "WaitForTransactionReceiptError",
)<{
  cause: WaitForTransactionReceiptErrorType
}> {}

export class CreatePublicClientError extends Data.TaggedError("CreatePublicClientError")<{
  cause: CreatePublicClientErrorType
}> {}

export class SendTransactionError extends Data.TaggedError("SendTransactionError")<{
  cause: SendTransactionErrorType
}> {}

export type SwitchChainError = EvmSwitchChainError | CosmosSwitchChainError

export class ConnectorClientError extends Data.TaggedError("ConnectorClientError")<{
  wagmiConfig: unknown
  cause: GetConnectorClientErrorType
}> {}
export class AmountError extends Data.TaggedError("AmountError")<{
  message: string
}> {}

export type SubmitTransferError = SendTransactionError | CreateWalletClientError
