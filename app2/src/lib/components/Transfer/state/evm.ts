import { Data, Duration, Effect, type Exit, Schedule } from "effect"
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

export type EffectToExit<T> = T extends Effect.Effect<infer A, infer E, any>
  ? Exit.Exit<A, E>
  : never

export type TransactionSubmissionEvm = Data.TaggedEnum<{
  Filling: {}
  SwitchChainInProgress: {}
  SwitchChainComplete: { exit: EffectToExit<ReturnType<typeof switchChain>> }
  WriteContractInProgress: {}
  WriteContractComplete: { exit: EffectToExit<ReturnType<typeof writeContract>> }
  TransactionReceiptInProgress: { readonly hash: Hash }
  TransactionReceiptComplete: { exit: EffectToExit<ReturnType<typeof waitForTransactionReceipt>> }
}>

export const TransactionSubmissionEvm = Data.taggedEnum<TransactionSubmissionEvm>()
const {
  SwitchChainInProgress,
  SwitchChainComplete,
  WriteContractInProgress,
  WriteContractComplete,
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
    SwitchChainInProgress: async () =>
      SwitchChainComplete({
        exit: await Effect.runPromiseExit(switchChain(chain))
      }),
    SwitchChainComplete: ({ exit }) =>
      exit._tag === "Failure" ? SwitchChainInProgress() : WriteContractInProgress(),
    WriteContractInProgress: async () => {
      const retryableWrite = writeContract(walletClient, params).pipe(
        Effect.retry(
          Schedule.exponential(Duration.millis(100)).pipe(Schedule.intersect(Schedule.recurs(5)))
        )
      )
      return WriteContractComplete({
        exit: await Effect.runPromiseExit(retryableWrite)
      })
    },
    WriteContractComplete: ({ exit }) =>
      exit._tag === "Failure"
        ? WriteContractInProgress()
        : TransactionReceiptInProgress({ hash: exit.value }),
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
