import { Effect } from "effect"
import type { Hash, WaitForTransactionReceiptErrorType, WriteContractErrorType } from "viem"
import { WaitForTransactionReceiptError, WriteContractError } from "./errors.ts"
import { getPublicClient, getWalletClient } from "./clients.ts"
import type { TransactionEvmParams } from "$lib/services/transfer-ucs03-evm/machine"
import { getAccount } from "$lib/services/transfer-ucs03-evm/account.ts"
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts"
import { generateSalt } from "./salt.ts"

export const submitTransfer = (transactionArgs: TransactionEvmParams) =>
  Effect.gen(function* () {
    const walletClient = yield* getWalletClient

    console.log("submit", transactionArgs)

    const account = yield* Effect.flatMap(getAccount, account =>
      account ? Effect.succeed(account) : Effect.fail(new Error("No account connected"))
    )

    const salt = yield* generateSalt

    const hash = yield* Effect.tryPromise({
      try: () => {
        return walletClient.writeContract({
          account: account.address as `0x${string}`,
          abi: ucs03ZkgmAbi,
          chain: transactionArgs.chain,
          functionName: "transferV2",
          address: transactionArgs.ucs03address,
          value: BigInt(0.0080085 * 10 ** 18),
          args: [
            transactionArgs.sourceChannelId,
            transactionArgs.receiver,
            transactionArgs.baseToken,
            transactionArgs.baseAmount,
            transactionArgs.quoteToken,
            transactionArgs.quoteAmount,
            transactionArgs.timeoutHeight,
            transactionArgs.timeoutTimestamp,
            salt,
            transactionArgs.wethQuoteToken
          ]
        })
      },
      catch: err => new WriteContractError({ cause: err as WriteContractErrorType })
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
