import { Data, Effect } from "effect"
import type {
  CreateWalletClientErrorType,
  SendTransactionErrorType,
  SendTransactionParameters
} from "viem"
import { createWalletClient } from "viem"
import { sepolia } from "viem/chains"

export class CreateWalletClientError extends Data.TaggedError("CreateWalletClientError")<{
  cause: CreateWalletClientErrorType
}> {}

export class SendTransactionError extends Data.TaggedError("SendTransactionError")<{
  cause: SendTransactionErrorType
}> {}

export type SubmitTransferError = SendTransactionError | CreateWalletClientError

export const submitTransfer = (transactionArgs: SendTransactionParameters) =>
  Effect.gen(function* () {
    const walletClient = yield* Effect.try({
      try: () =>
        createWalletClient({
          chain: sepolia,
          transport: () => window.ethereum
        }),
      catch: err => new CreateWalletClientError({ cause: err as CreateWalletClientErrorType })
    })

    const hash = yield* Effect.tryPromise({
      try: () => walletClient.sendTransaction(transactionArgs),
      catch: err => new SendTransactionError({ cause: err as SendTransactionErrorType })
    })
    return hash
  })
