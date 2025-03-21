import { Effect } from "effect"
import type { Account, Hash, WaitForTransactionReceiptErrorType } from "viem"
import { WaitForTransactionReceiptError } from "./errors.ts"
import { getPublicClient, getWalletClient } from "../evm/clients.ts"
import { getAccount } from "$lib/services/transfer-ucs03-evm/account.ts"
import type { Chain } from "$lib/schema/chain.ts"
import type { ValidTransfer } from "$lib/schema/transfer-args.ts"
import { generateSalt } from "$lib/services/shared"
import { sepolia } from "viem/chains"
import { Batch, FungibleAssetOrder, encodeAbi, sendInstructionEvm } from "@unionlabs/sdk/ucs03"
import { ucs03abi } from "@unionlabs/sdk/evm/abi"
import { readErc20Meta } from "@unionlabs/sdk/evm/erc20"
import { ViemPublicClient, ViemWalletClient, writeContract } from "@unionlabs/sdk/evm"

export const submitTransfer = (chain: Chain, transfer: ValidTransfer["args"]) =>
  Effect.gen(function* () {
    if (transfer.sourceRpcType !== "evm") {
      return yield* Effect.fail(new Error("Only EVM transfers are supported"))
    }
    const account = yield* Effect.flatMap(getAccount, account =>
      account ? Effect.succeed(account) : Effect.fail(new Error("No account connected"))
    )
    const salt = yield* generateSalt

    const client = yield* getPublicClient(chain)
    const onchainBaseTokenMeta = yield* readErc20Meta(transfer.baseToken).pipe(
      Effect.provideService(ViemPublicClient, { client })
    )

    const walletClient = yield* getWalletClient(chain)

    return "0x0fabab"

    // return yield* sendInstructionEvm(
    //   Batch([
    //     FungibleAssetOrder([
    //       account.address as `0x${string}`,
    //       transfer.receiver as `0x${string}`,
    //       transfer.baseToken,
    //       transfer.baseAmount,
    //       onchainBaseTokenMeta.symbol,
    //       onchainBaseTokenMeta.name,
    //       onchainBaseTokenMeta.decimals,
    //       9n, // when unwrapping, otherwise 0
    //       transfer.quoteToken,
    //       transfer.quoteAmount
    //     ])
    //   ])
    // ).pipe(
    //   Effect.provideService(ViemWalletClient, {
    //     account: account as Account,
    //     client: walletClient,
    //     chain: sepolia
    //   })
    // )
    // return yield* writeContract(walletClient, {
    //   account: account.address as `0x${string}`,
    //   abi: ucs03abi,
    //   chain: sepolia,
    //   functionName: "send",
    //   address: transfer.ucs03address as `0x${string}`,
    //   args: [
    //     transfer.sourceChannelId,
    //     transfer.timeoutHeight,
    //     BigInt(transfer.timeoutTimestamp),
    //     salt,
    //     encodeAbi(
    // Batch([
    //   FungibleAssetOrder([
    //     account.address as `0x${string}`,
    //     transfer.receiver as `0x${string}`,
    //     transfer.baseToken,
    //     transfer.baseAmount,
    //     onchainBaseTokenMeta.symbol,
    //     onchainBaseTokenMeta.name,
    //     onchainBaseTokenMeta.decimals,
    //     9n, // when unwrapping, otherwise 0
    //     transfer.quoteToken,
    //     transfer.quoteAmount
    //   ])
    // ])
    //     )
    //   ]
    // })
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
