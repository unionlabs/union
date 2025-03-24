import { Data, Effect, type Exit } from "effect"
import { switchChain } from "$lib/services/transfer-ucs03-evm"
import { ViemPublicClient, waitForTransactionReceipt, writeContract } from "@unionlabs/sdk/evm"
import type { Chain, Hash, PublicClient, WalletClient } from "viem"

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

export const evmNextState = async <P extends Parameters<typeof writeContract>[1]>(
  ts: TransactionSubmissionEvm,
  chain: Chain,
  publicClient: PublicClient,
  walletClient: WalletClient,
  params: P
): Promise<TransactionSubmissionEvm> =>
  TransactionSubmissionEvm.$match(ts, {
    Filling: () => SwitchChainInProgress(),
    SwitchChainInProgress: async () =>
      SwitchChainComplete({
        exit: await Effect.runPromiseExit(switchChain(chain))
      }),
    SwitchChainComplete: ({ exit }) =>
      exit._tag === "Failure" ? SwitchChainInProgress() : WriteContractInProgress(),
    WriteContractInProgress: async () =>
      WriteContractComplete({
        exit: await Effect.runPromiseExit(writeContract(walletClient, params))
      }),
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

export const isComplete = (state: TransactionSubmissionEvm) =>
  state._tag === "TransactionReceiptComplete" && state.exit._tag === "Success"
