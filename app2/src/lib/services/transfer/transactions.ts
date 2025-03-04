import { Effect } from "effect"
import type {
  Hash,
  SendTransactionErrorType,
  SendTransactionParameters,
  WaitForTransactionReceiptErrorType
} from "viem"
import { SendTransactionError, WaitForTransactionReceiptError } from "./errors.ts"
import { getPublicClient, getWalletClient } from "./clients.ts"

export const submitTransfer = (transactionArgs: SendTransactionParameters) =>
  Effect.gen(function* () {
    const walletClient = yield* getWalletClient

    const hash = yield* Effect.tryPromise({
      try: () => walletClient.sendTransaction(transactionArgs),
      catch: err => new SendTransactionError({ cause: err as SendTransactionErrorType })
    })

    return hash
  })

export const waitForReceipt = (hash: Hash) =>
  Effect.gen(function* () {
    const publicClient = yield* getPublicClient

    const receipt = yield* Effect.tryPromise({
      try: () => publicClient.waitForTransactionReceipt({ hash }),
      catch: err =>
        new WaitForTransactionReceiptError({ cause: err as WaitForTransactionReceiptErrorType })
    })

    return receipt
  })
