import { Data, Effect } from "effect"
import type { SendTransactionParameters } from "viem"
import { createWalletClient } from "viem"
import { sepolia } from "viem/chains"

export class CreateWalletClientError extends Data.TaggedError("CreateWalletClientError")<{
  cause: Error
}> {}

export class SendTransactionError extends Data.TaggedError("SendTransactionError")<{
  cause: Error
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
      catch: err => new CreateWalletClientError({ cause: err as Error })
    })

    const hash = yield* Effect.tryPromise({
      try: () => walletClient.sendTransaction(transactionArgs),
      catch: err => new SendTransactionError({ cause: err as Error })
    })
    return hash
  })
