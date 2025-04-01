import { Effect } from "effect"
import {
  erc20Abi,
  type Hash,
  type WaitForTransactionReceiptErrorType,
  type WriteContractErrorType
} from "viem"
import { WaitForTransactionReceiptError, WriteContractError } from "./errors.ts"
import { getPublicClient, getWalletClient } from "../evm/clients.ts"
import { getAccount } from "$lib/services/transfer-ucs03-evm/account.ts"
import type { Chain } from "@unionlabs/sdk/schema"
import type { ValidTransfer } from "@unionlabs/sdk/schema"

export const approveTransfer = (transfer: ValidTransfer["args"]) =>
  Effect.gen(function* () {
    const walletClient = yield* getWalletClient(transfer.sourceChain)

    const account = yield* Effect.flatMap(getAccount, account =>
      account ? Effect.succeed(account) : Effect.fail(new Error("No account connected"))
    )

    const hash = yield* Effect.tryPromise({
      try: () =>
        walletClient.writeContract({
          account: account.address as `0x${string}`,
          abi: erc20Abi,
          functionName: "approve",
          address: transfer.baseToken,
          args: [transfer.ucs03address as `0x${string}`, transfer.baseAmount]
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
