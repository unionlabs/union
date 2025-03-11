import { Effect } from "effect"
import type { Hash, WaitForTransactionReceiptErrorType, WriteContractErrorType } from "viem"
import { WaitForTransactionReceiptError, WriteContractError } from "./errors.ts"
import { getPublicClient, getWalletClient } from "../evm/clients.ts"
import { getAccount } from "$lib/services/transfer-ucs03-evm/account.ts"
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts"
import { generateSalt } from "./salt.ts"
import type { Chain } from "$lib/schema/chain.ts"
import type { ValidTransfer } from "$lib/schema/transfer-args.ts"

export const submitTransfer = (chain: Chain, transfer: ValidTransfer) =>
  Effect.gen(function* () {
    const walletClient = yield* getWalletClient(chain)

    const account = yield* Effect.flatMap(getAccount, account =>
      account ? Effect.succeed(account) : Effect.fail(new Error("No account connected"))
    )

    const salt = yield* generateSalt

    return yield* Effect.tryPromise({
      try: () => {
        return walletClient.writeContract({
          account: account.address as `0x${string}`,
          abi: ucs03ZkgmAbi,
          chain: transfer.args.sourceChain,
          functionName: "transferV2",
          address: transfer.args.ucs03address,
          value: BigInt(0.0080085 * 10 ** 18),
          args: [
            transfer.args.sourceChannelId,
            transfer.args.receiver,
            transfer.args.baseToken,
            transfer.args.baseAmount,
            transfer.args.quoteToken,
            transfer.args.quoteAmount,
            transfer.args.timeoutHeight,
            transfer.args.timeoutTimestamp,
            salt,
            transfer.args.wethQuoteToken
          ]
        })
      },
      catch: err => new WriteContractError({ cause: err as WriteContractErrorType })
    })
  })

export const waitForTransferReceipt = (chain: Chain, hash: Hash) =>
  Effect.gen(function* () {
    const publicClient = yield* getPublicClient(chain)
    return yield* Effect.tryPromise({
      try: () => publicClient.waitForTransactionReceipt({ hash }),
      catch: err =>
        new WaitForTransactionReceiptError({ cause: err as WaitForTransactionReceiptErrorType })
    })
  })
