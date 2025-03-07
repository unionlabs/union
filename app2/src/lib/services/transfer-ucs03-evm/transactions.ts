import { Effect } from "effect"
import type { Hash, WaitForTransactionReceiptErrorType, WriteContractErrorType } from "viem"
import { WaitForTransactionReceiptError, WriteContractError } from "./errors.ts"
import { getPublicClient, getWalletClient } from "../evm/clients.ts"
import type { Ucs03TransferEvm } from "$lib/services/transfer-ucs03-evm/machine"
import { getAccount } from "$lib/services/transfer-ucs03-evm/account.ts"
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts"
import { generateSalt } from "./salt.ts"
import type { Chain } from "$lib/schema/chain.ts"

export const submitTransfer = (chain: Chain, transactionArgs: Ucs03TransferEvm) =>
  Effect.gen(function* () {
    const walletClient = yield* getWalletClient(chain)

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
          chain: transactionArgs.sourceChain,
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

export const waitForTransferReceipt = (chain: Chain, hash: Hash) =>
  Effect.gen(function* () {
    const publicClient = yield* getPublicClient(chain)

    const receipt = yield* Effect.tryPromise({
      try: () => publicClient.waitForTransactionReceipt({ hash }),
      catch: err =>
        new WaitForTransactionReceiptError({ cause: err as WaitForTransactionReceiptErrorType })
    })

    return receipt
  })
