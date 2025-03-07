import { Effect } from "effect"
import type {
  Hash,
  SendTransactionErrorType,
  SendTransactionParameters,
  WaitForTransactionReceiptErrorType
} from "viem"
import { SendTransactionError, WaitForTransactionReceiptError } from "./errors.ts"
import { getPublicClient, getWalletClient } from "../evm/clients.ts"
import type { Chain } from "$lib/schema/chain.ts"

export const submitTransfer = (chain: Chain, transactionArgs: SendTransactionParameters) =>
  Effect.gen(function* () {
    const walletClient = yield* getWalletClient(chain)

    const hash = yield* Effect.tryPromise({
      try: () => walletClient.sendTransaction(transactionArgs),
      catch: err => new SendTransactionError({ cause: err as SendTransactionErrorType })
    })

    return hash
  })

export const waitForReceipt = (chain: Chain, hash: Hash) =>
  Effect.gen(function* () {
    const publicClient = yield* getPublicClient(chain)

    const receipt = yield* Effect.tryPromise({
      try: () => publicClient.waitForTransactionReceipt({ hash }),
      catch: err =>
        new WaitForTransactionReceiptError({ cause: err as WaitForTransactionReceiptErrorType })
    })

    return receipt
  })
