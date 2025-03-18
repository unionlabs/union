import { Effect } from "effect"
import {
  encodeAbiParameters,
  type Hash,
  type WaitForTransactionReceiptErrorType,
  type WriteContractErrorType
} from "viem"
import { WaitForTransactionReceiptError, WriteContractError } from "./errors.ts"
import { getPublicClient, getWalletClient } from "../evm/clients.ts"
import { getAccount } from "$lib/services/transfer-ucs03-evm/account.ts"
import { ucs03ZkgmAbi } from "$lib/abi/ucs03.ts"
import type { Chain } from "$lib/schema/chain.ts"
import type { ValidTransfer } from "$lib/schema/transfer-args.ts"
import { generateSalt } from "$lib/services/shared"
import { sepolia } from "viem/chains"
import { fetchErc20Decimals, fetchErc20Name, fetchErc20Symbol } from "../evm/erc20.ts"
import { Batch, FungibleAssetOrder } from "@unionlabs/sdk/evm/ucs03"
import { ucs03abi } from "@unionlabs/sdk/evm/abi"

export const submitTransfer = (chain: Chain, transfer: ValidTransfer["args"]) =>
  Effect.gen(function* () {
    if (transfer.sourceRpcType !== "evm") {
      return yield* Effect.fail(new Error("Only EVM transfers are supported"))
    }
    const walletClient = yield* getWalletClient(chain)
    const account = yield* Effect.flatMap(getAccount, account =>
      account ? Effect.succeed(account) : Effect.fail(new Error("No account connected"))
    )
    const salt = yield* generateSalt

    const client = yield* getPublicClient(chain)

    const onchainBaseTokenSymbol = yield* fetchErc20Symbol({
      client,
      tokenAddress: transfer.baseToken
    })
    const onchainBaseTokenName = yield* fetchErc20Name({
      client,
      tokenAddress: transfer.baseToken
    })
    const onchainBaseTokenDecimals = yield* fetchErc20Decimals({
      client,
      tokenAddress: transfer.baseToken
    })

    console.log({
      account: account.address as `0x${string}`,
      abi: ucs03abi,
      chain: sepolia,
      functionName: "send",
      address: transfer.ucs03address as `0x${string}`,
      args: [
        transfer.sourceChannelId,
        transfer.timeoutHeight,
        BigInt(transfer.timeoutTimestamp),
        salt,
        Batch([
          FungibleAssetOrder([
            account.address as `0x${string}`,
            transfer.receiver as `0x${string}`,
            transfer.baseToken,
            transfer.baseAmount,
            onchainBaseTokenSymbol,
            onchainBaseTokenName,
            onchainBaseTokenDecimals,
            0n,
            transfer.quoteToken,
            transfer.quoteAmount
          ]),
          FungibleAssetOrder([
            account.address as `0x${string}`,
            transfer.receiver as `0x${string}`,
            transfer.baseToken,
            transfer.baseAmount,
            onchainBaseTokenSymbol,
            onchainBaseTokenName,
            onchainBaseTokenDecimals,
            0n,
            transfer.quoteToken,
            transfer.quoteAmount
          ])
        ])
      ]
    })

    return yield* Effect.tryPromise({
      try: () => {
        return walletClient.writeContract({
          account: account.address as `0x${string}`,
          abi: ucs03abi,
          chain: sepolia,
          functionName: "send",
          address: transfer.ucs03address as `0x${string}`,
          args: [
            transfer.sourceChannelId,
            transfer.timeoutHeight,
            BigInt(transfer.timeoutTimestamp),
            salt,
            Batch([
              FungibleAssetOrder([
                account.address as `0x${string}`,
                transfer.receiver as `0x${string}`,
                transfer.baseToken,
                transfer.baseAmount,
                onchainBaseTokenSymbol,
                onchainBaseTokenName,
                onchainBaseTokenDecimals,
                0n,
                transfer.quoteToken,
                transfer.quoteAmount
              ]),
              FungibleAssetOrder([
                account.address as `0x${string}`,
                transfer.receiver as `0x${string}`,
                transfer.baseToken,
                transfer.baseAmount,
                onchainBaseTokenSymbol,
                onchainBaseTokenName,
                onchainBaseTokenDecimals,
                0n,
                transfer.quoteToken,
                transfer.quoteAmount
              ])
            ])
          ]
        })
      },
      catch: err => {
        console.error("write contract error", err)
        return new WriteContractError({ cause: err as WriteContractErrorType })
      }
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
