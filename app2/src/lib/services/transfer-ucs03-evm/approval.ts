import { Effect } from "effect"
import {
  erc20Abi,
  getAddress,
  type Hash,
  type SendTransactionErrorType,
  type WaitForTransactionReceiptErrorType,
  type WriteContractErrorType
} from "viem"
import { WaitForTransactionReceiptError, WriteContractError } from "./errors.ts"
import { getPublicClient, getWalletClient } from "./clients.ts"
import type { TransactionEvmParams } from "$lib/services/transfer-ucs03-evm/machine"
import { getAccount } from "$lib/services/transfer-ucs03-evm/account.ts"

export const approveTransfer = (transactionArgs: TransactionEvmParams) =>
  Effect.gen(function* () {
    const walletClient = yield* getWalletClient

    const account = yield* Effect.flatMap(getAccount, account =>
      account ? Effect.succeed(account) : Effect.fail(new Error("No account connected"))
    )

    const hash = yield* Effect.tryPromise({
      try: () =>
        walletClient.writeContract({
          account: account.address as `0x${string}`,
          abi: erc20Abi,
          chain: transactionArgs.chain,
          functionName: "approve",
          address: transactionArgs.baseToken,
          args: [transactionArgs.ucs03address, transactionArgs.baseAmount]
        }),
      catch: err => new WriteContractError({ cause: err as WriteContractErrorType })
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
