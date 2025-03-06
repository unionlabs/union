import { Effect } from "effect"
import {
  erc20Abi,
  getAddress,
  type Hash,
  type SendTransactionErrorType,
  type WaitForTransactionReceiptErrorType
} from "viem"
import { SendTransactionError, WaitForTransactionReceiptError } from "./errors.ts"
import { getPublicClient, getWalletClient } from "./clients.ts"
import type { TransactionEvmParams } from "$lib/services/transfer-ucs03-evm/machine"

export const approveTransfer = (transactionArgs: TransactionEvmParams) =>
  Effect.gen(function* () {
    const walletClient = yield* getWalletClient

    const hash = yield* Effect.tryPromise({
      try: () =>
        walletClient.writeContract({
          account: transactionArgs.account,
          abi: erc20Abi,
          chain: transactionArgs.chain,
          functionName: "approve",
          address: getAddress(transactionArgs.args.baseToken),
          args: [getAddress(transactionArgs.address), transactionArgs.args.baseAmount]
        }),
      catch: err => new SendTransactionError({ cause: err as SendTransactionErrorType })
    })

    return hash
  })

export const waitForApprovalReceipt = (hash: Hash) =>
  Effect.gen(function* () {
    const publicClient = yield* getPublicClient

    const receipt = yield* Effect.tryPromise({
      try: () => publicClient.waitForTransactionReceipt({ hash }),
      catch: err =>
        new WaitForTransactionReceiptError({ cause: err as WaitForTransactionReceiptErrorType })
    })

    return receipt
  })
