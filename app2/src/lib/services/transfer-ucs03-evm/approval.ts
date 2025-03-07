import { Effect } from "effect"
import {
  erc20Abi,
  type Hash,
  type WaitForTransactionReceiptErrorType,
  type WriteContractErrorType
} from "viem"
import { WaitForTransactionReceiptError, WriteContractError } from "./errors.ts"
import { getPublicClient, getWalletClient } from "../evm/clients.ts"
import type { Ucs03TransferEvm } from "$lib/services/transfer-ucs03-evm/machine"
import { getAccount } from "$lib/services/transfer-ucs03-evm/account.ts"
import type { Chain } from "$lib/schema/chain.ts"

export const approveTransfer = (chain: Chain, transactionArgs: Ucs03TransferEvm) =>
  Effect.gen(function* () {
    const walletClient = yield* getWalletClient(chain)

    const account = yield* Effect.flatMap(getAccount, account =>
      account ? Effect.succeed(account) : Effect.fail(new Error("No account connected"))
    )

    const hash = yield* Effect.tryPromise({
      try: () =>
        walletClient.writeContract({
          account: account.address as `0x${string}`,
          abi: erc20Abi,
          functionName: "approve",
          address: transactionArgs.baseToken,
          args: [transactionArgs.ucs03address, transactionArgs.baseAmount]
        }),
      catch: err => new WriteContractError({ cause: err as WriteContractErrorType })
    })

    return hash
  })

export const waitForApprovalReceipt = (chain: Chain, hash: Hash) =>
  Effect.gen(function* () {
    const publicClient = yield* getPublicClient(chain)

    const receipt = yield* Effect.tryPromise({
      try: () => publicClient.waitForTransactionReceipt({ hash }),
      catch: err =>
        new WaitForTransactionReceiptError({ cause: err as WaitForTransactionReceiptErrorType })
    })

    return receipt
  })
