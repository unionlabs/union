import { Data, Effect, type Exit } from "effect"
import { switchChain } from "$lib/services/transfer-ucs03-evm"
import { ViemPublicClient, waitForTransactionReceipt, writeContract } from "@unionlabs/sdk/evm"
import type {
  Abi,
  Chain,
  ContractFunctionArgs,
  ContractFunctionName,
  Hash,
  PublicClient,
  WalletClient,
  WriteContractParameters
} from "viem"
import { getLastConnectedWalletId } from "$lib/wallet/evm/config.svelte.ts"
import { resolveSafeTx } from "$lib/transfer/shared/services/handlers/safe-hash.ts"

export type EffectToExit<T> = T extends Effect.Effect<infer A, infer E, any>
  ? Exit.Exit<A, E>
  : never

export type TransactionSubmissionEvm = Data.TaggedEnum<{
  Filling: {}
  SwitchChainInProgress: {}
  SwitchChainComplete: { exit: EffectToExit<ReturnType<typeof switchChain>> }
  WriteContractInProgress: {}
  WriteContractComplete: { exit: EffectToExit<ReturnType<typeof writeContract>> }
  WaitForSafeWalletHash: { readonly hash: Hash } // the safeTxHash
  TransactionReceiptInProgress: { readonly hash: Hash } // on chain hash
  TransactionReceiptComplete: { exit: EffectToExit<ReturnType<typeof waitForTransactionReceipt>> }
}>

export const TransactionSubmissionEvm = Data.taggedEnum<TransactionSubmissionEvm>()
const {
  SwitchChainInProgress,
  SwitchChainComplete,
  WriteContractInProgress,
  WriteContractComplete,
  WaitForSafeWalletHash,
  TransactionReceiptInProgress,
  TransactionReceiptComplete
} = TransactionSubmissionEvm

export const nextStateEvm = async <
  TAbi extends Abi,
  TFunctionName extends ContractFunctionName<TAbi, "nonpayable" | "payable"> = ContractFunctionName<
    TAbi,
    "nonpayable" | "payable"
  >,
  TArgs extends ContractFunctionArgs<
    TAbi,
    "nonpayable" | "payable",
    TFunctionName
  > = ContractFunctionArgs<TAbi, "nonpayable" | "payable", TFunctionName>
>(
  ts: TransactionSubmissionEvm,
  chain: Chain,
  publicClient: PublicClient,
  walletClient: WalletClient,
  params: WriteContractParameters<TAbi, TFunctionName, TArgs>
): Promise<TransactionSubmissionEvm> =>
  TransactionSubmissionEvm.$match(ts, {
    Filling: () => SwitchChainInProgress(),

    SwitchChainInProgress: async () => {
      const isSafeWallet = getLastConnectedWalletId() === "safe" // safe wagmi connector does not support wagmiSwitchChain
      return isSafeWallet
        ? WriteContractInProgress()
        : SwitchChainComplete({
            exit: await Effect.runPromiseExit(switchChain(chain))
          })
    },

    SwitchChainComplete: ({ exit }) =>
      exit._tag === "Failure" ? SwitchChainInProgress() : WriteContractInProgress(),

    WriteContractInProgress: async () =>
      WriteContractComplete({
        exit: await Effect.runPromiseExit(writeContract(walletClient, params))
      }),

    WriteContractComplete: async ({ exit }) => {
      if (exit._tag === "Failure") {
        return WriteContractInProgress()
      }

      const wallet = getLastConnectedWalletId()
      const hash = exit.value

      return wallet === "safe" // needed due to safe wagmi connector returns safeTx hash and not the onchain one
        ? WaitForSafeWalletHash({ hash })
        : TransactionReceiptInProgress({ hash })
    },

    WaitForSafeWalletHash: async ({ hash }) => {
      const resolvedExit = await Effect.runPromiseExit(resolveSafeTx(hash)) // TODO

      return resolvedExit._tag === "Failure"
        ? WaitForSafeWalletHash({ hash })
        : TransactionReceiptInProgress({ hash: resolvedExit.value })
    },

    TransactionReceiptInProgress: async ({ hash }) =>
      TransactionReceiptComplete({
        exit: await Effect.runPromiseExit(
          waitForTransactionReceipt(hash).pipe(
            Effect.provideService(ViemPublicClient, { client: publicClient })
          )
        )
      }),

    TransactionReceiptComplete: () => ts
  })

export const hasFailedExit = (state: TransactionSubmissionEvm) =>
  "exit" in state && state.exit._tag === "Failure"

export const isComplete = (state: TransactionSubmissionEvm): string | false => {
  if (state._tag === "TransactionReceiptComplete" && state.exit._tag === "Success") {
    return state.exit.value.transactionHash
  }
  return false
}
