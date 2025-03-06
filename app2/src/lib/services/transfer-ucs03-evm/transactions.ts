import { Effect } from "effect"
import type { Hash, SendTransactionErrorType, WaitForTransactionReceiptErrorType } from "viem"
import { SendTransactionError, WaitForTransactionReceiptError } from "./errors.ts"
import { getPublicClient, getWalletClient } from "./clients.ts"
import type { TransactionEvmParams } from "$lib/services/transfer-ucs03-evm/machine"
import { getAccount } from "$lib/services/transfer-ucs03-evm/account.ts"
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts"

export const submitTransfer = (transactionArgs: TransactionEvmParams) =>
  Effect.gen(function* () {
    const walletClient = yield* getWalletClient

    console.log("submit", transactionArgs)

    const account = yield* Effect.flatMap(getAccount, account =>
      account ? Effect.succeed(account) : Effect.fail(new Error("No account connected"))
    )

    const hash = yield* Effect.tryPromise({
      try: () => {
        return walletClient.writeContract({
          account: account.address as `0x${string}`,
          abi: ucs03ZkgmAbi,
          chain: transactionArgs.chain,
          functionName: "transferV2",
          address: transactionArgs.address,
          value: BigInt(0.0080085 * 10 ** 18),
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
