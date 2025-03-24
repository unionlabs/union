import {Data, Effect, Exit} from "effect";
import {switchChain, type waitForTransferReceipt} from "$lib/services/transfer-ucs03-evm";
import {ViemPublicClient, waitForTransactionReceipt, writeContract} from "@unionlabs/sdk/evm";
import type {Chain, Hash, PublicClient, WalletClient} from "viem";

export type EffectToExit<T> = T extends Effect.Effect<infer A, infer E, any> ? Exit.Exit<A, E> : never

export type SwitchChainState = Data.TaggedEnum<{
  InProgress: {}
  Complete: { exit: EffectToExit<ReturnType<typeof switchChain>> } // TODO: yield requirements in SwitchChain
}>
export const SwitchChainState = Data.taggedEnum<SwitchChainState>()

export type WriteContractState = Data.TaggedEnum<{
  InProgress: {}
  Complete: { exit: EffectToExit<ReturnType<typeof writeContract>> }
}>
export const WriteContractState = Data.taggedEnum<WriteContractState>()

export type TransactionReceiptState = Data.TaggedEnum<{
  InProgress: { readonly hash: Hash }
  Complete: { exit: EffectToExit<ReturnType<typeof waitForTransferReceipt>> }
}>
export const TransactionReceiptState = Data.taggedEnum<TransactionReceiptState>()

export type TransactionSubmissionEvm = Data.TaggedEnum<{
  Filling: {}
  SwitchChain: { state: SwitchChainState }
  WriteContract: { state: WriteContractState }
  TransactionReceipt: { state: TransactionReceiptState }
}>

export type StateWithExit =
  | { _tag: "SwitchChain"; state: SwitchChainState }
  | { _tag: "WriteContract"; state: WriteContractState }
  | { _tag: "TransactionReceipt"; state: TransactionReceiptState }

export const TransactionSubmissionEvm = Data.taggedEnum<TransactionSubmissionEvm>()

export const evmNextState = async <P extends Parameters<typeof writeContract>[1]>(
  ts: TransactionSubmissionEvm,
  chain: Chain,
  publicClient: PublicClient,
  walletClient: WalletClient,
  params: P
): Promise<TransactionSubmissionEvm> =>
  TransactionSubmissionEvm.$match(ts, {
    Filling: () => TransactionSubmissionEvm.SwitchChain({ state: SwitchChainState.InProgress() }),
    SwitchChain: ({ state }) =>
      SwitchChainState.$match(state, {
        InProgress: async () => {
          const exit = await Effect.runPromiseExit(switchChain(chain))
          return TransactionSubmissionEvm.SwitchChain({
            state: SwitchChainState.Complete({ exit })
          })
        },
        Complete: ({ exit }) =>
          exit._tag === "Failure"
            ? TransactionSubmissionEvm.SwitchChain({ state: SwitchChainState.InProgress() })
            : TransactionSubmissionEvm.WriteContract({ state: WriteContractState.InProgress() })
      }),
    WriteContract: ({ state }) =>
      WriteContractState.$match(state, {
        InProgress: async () => {
          const exit = await Effect.runPromiseExit(writeContract(walletClient, params))
          return TransactionSubmissionEvm.WriteContract({
            state: WriteContractState.Complete({ exit })
          })
        },
        Complete: ({ exit }) =>
          exit._tag === "Failure"
            ? TransactionSubmissionEvm.WriteContract({ state: WriteContractState.InProgress() })
            : TransactionSubmissionEvm.TransactionReceipt({
              state: TransactionReceiptState.InProgress({ hash: exit.value })
            })
      }),
    TransactionReceipt: ({ state }) =>
      TransactionReceiptState.$match(state, {
        InProgress: async ({ hash }) => {
          const exit = await Effect.runPromiseExit(
            waitForTransactionReceipt(hash).pipe(
              Effect.provideService(ViemPublicClient, { client: publicClient })
            )
          )
          return TransactionSubmissionEvm.TransactionReceipt({
            state: TransactionReceiptState.Complete({ exit })
          })
        },
        Complete: () => ts // There is no next state, return self
      })
  })

export const hasFailedExit = (state: TransactionSubmissionEvm) =>
  state._tag !== "Filling" && state.state._tag === "Complete" && state.state.exit._tag === "Failure"

export const isComplete = (state: TransactionSubmissionEvm) =>
  state._tag === "TransactionReceipt" &&
  state.state._tag === "Complete" &&
  state.state.exit._tag === "Success"
