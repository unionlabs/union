import { Effect } from "effect"
import type { Hash, WaitForTransactionReceiptErrorType, WriteContractErrorType } from "viem"
import { WaitForTransactionReceiptError, WriteContractError } from "./errors.ts"
import { getPublicClient } from "../aptos/clients.ts"
import { getAccount } from "$lib/services/transfer-ucs03-aptos/account.ts"
import { generateSalt } from "./salt.ts"
import type { Chain } from "$lib/schema/chain.ts"
import type { ValidTransfer } from "$lib/schema/transfer-args.ts"

export const submitTransferAptos = (chain: Chain, transfer: ValidTransfer["args"]) =>
  Effect.gen(function* () {
    if (transfer.sourceRpcType !== "aptos") {
      return yield* Effect.fail(new Error("Only EVM transfers are supported"))
    }
    const account = yield* Effect.flatMap(getAccount, account =>
      account ? Effect.succeed(account) : Effect.fail(new Error("No account connected"))
    )
    const salt = yield* generateSalt

    const walletPayload = {
      function: `${transfer.ucs03address}::ibc_app::transfer`,
      type_arguments: [],
      arguments: [
        transfer.sourceChannelId,
        hexToAscii(transfer.receiver), // It is hexing again in it.
        transfer.baseToken,
        transfer.baseAmount.toString(),
        hexToAscii(transfer.quoteToken), // It is hexing again in it.
        transfer.quoteAmount.toString(),
        18446744073709551615n.toString(), // TODO: Check this value, use transfer.timeoutHeight later
        18446744073709551615n.toString(), // TODO: Check this value, use transfer.timeoutTimestamp later
        salt
      ]
    }

    return yield* Effect.tryPromise({
      try: () => {
        return account.signAndSubmitTransaction({ payload: walletPayload })
      },
      catch: err => new WriteContractError({ cause: err as WriteContractErrorType })
    })
  })

export const waitForTransferReceiptAptos = (chain: Chain, hash: Hash) =>
  Effect.gen(function* () {
    const publicClient = yield* getPublicClient(chain)
    return yield* Effect.tryPromise({
      try: () =>
        publicClient.waitForTransaction({
          transactionHash: hash,
          options: { checkSuccess: false }
        }),
      catch: err =>
        new WaitForTransactionReceiptError({ cause: err as WaitForTransactionReceiptErrorType })
    })
  })

function hexToAscii(hexString: string): string {
  // Remove the "0x" prefix if present.
  if (hexString.startsWith("0x") || hexString.startsWith("0X")) {
    hexString = hexString.slice(2)
  }
  let ascii = ""
  for (let i = 0; i < hexString.length; i += 2) {
    ascii += String.fromCharCode(Number.parseInt(hexString.substr(i, 2), 16))
  }
  return ascii
}
