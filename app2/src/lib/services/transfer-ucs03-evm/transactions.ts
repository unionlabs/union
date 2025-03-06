import { Effect } from "effect"
import type { Hash, SendTransactionErrorType, WaitForTransactionReceiptErrorType } from "viem"
import { SendTransactionError, WaitForTransactionReceiptError } from "./errors.ts"
import { getPublicClient, getWalletClient } from "./clients.ts"
import type { TransactionEvmParams } from "$lib/services/transfer-ucs03-evm/machine"

export const submitTransfer = (transactionArgs: TransactionEvmParams) =>
  Effect.gen(function* () {
    const walletClient = yield* getWalletClient

    console.log("submit", transactionArgs)

    const hash = yield* Effect.tryPromise({
      try: () => {
        if (transactionArgs.functionName === "transferV2") {
          return walletClient.writeContract({
            account: transactionArgs.account,
            abi: transactionArgs.abi,
            chain: transactionArgs.chain,
            functionName: transactionArgs.functionName,
            address: transactionArgs.address,
            value: transactionArgs.value,
            args: [
              transactionArgs.args.sourceChannelId,
              transactionArgs.args.receiver,
              transactionArgs.args.baseToken,
              transactionArgs.args.baseAmount,
              transactionArgs.args.quoteToken,
              transactionArgs.args.quoteAmount,
              transactionArgs.args.timeoutHeight,
              transactionArgs.args.timeoutTimestamp,
              transactionArgs.args.salt,
              transactionArgs.args.wethQuoteToken
            ]
          })
        }

        // For other functions, we need to handle them specifically too
        // since we can't pass transactionArgs directly due to type issues
        throw new Error(
          `Function ${transactionArgs.functionName} not implemented in submitTransfer`
        )
      },
      catch: err => new SendTransactionError({ cause: err as SendTransactionErrorType })
    })

    return hash
  })

export const waitForTransferReceipt = (hash: Hash) =>
  Effect.gen(function* () {
    const publicClient = yield* getPublicClient

    const receipt = yield* Effect.tryPromise({
      try: () => publicClient.waitForTransactionReceipt({ hash }),
      catch: err =>
        new WaitForTransactionReceiptError({ cause: err as WaitForTransactionReceiptErrorType })
    })

    return receipt
  })
