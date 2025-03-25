import {Data, Effect, type Exit} from "effect"
import {switchChain} from "$lib/services/transfer-ucs03-cosmos"
import {executeContract} from "@unionlabs/sdk/cosmos";
import type {Chain} from "$lib/schema/chain.ts";
import type {SigningCosmWasmClient} from "@cosmjs/cosmwasm-stargate";

export type EffectToExit<T> = T extends Effect.Effect<infer A, infer E, any>
  ? Exit.Exit<A, E>
  : never

export type TransactionSubmissionCosmos = Data.TaggedEnum<{
  Filling: {}
  SwitchChainInProgress: {}
  SwitchChainComplete: { exit: EffectToExit<ReturnType<typeof switchChain>> }
  WriteContractInProgress: {}
  WriteContractComplete: { exit: EffectToExit<ReturnType<typeof executeContract>> }
}>

export const TransactionSubmissionCosmos = Data.taggedEnum<TransactionSubmissionCosmos>()
const {
  SwitchChainInProgress,
  SwitchChainComplete,
  WriteContractInProgress,
  WriteContractComplete,
} = TransactionSubmissionCosmos

export const nextStateCosmos = async (
  ts: TransactionSubmissionCosmos,
  chain: Chain,
  signingClient: SigningCosmWasmClient,
  senderAddress: string,
  contractAddress: string,
  msg: Record<string, unknown>,
  funds?: ReadonlyArray<{ denom: string; amount: string }>,
): Promise<TransactionSubmissionCosmos> =>
  TransactionSubmissionCosmos.$match(ts, {
    Filling: () => SwitchChainInProgress(),
    SwitchChainInProgress: async () =>
      SwitchChainComplete({
        exit: await Effect.runPromiseExit(switchChain(chain))
      }),
    SwitchChainComplete: ({exit}) =>
      exit._tag === "Failure" ? SwitchChainInProgress() : WriteContractInProgress(),
    WriteContractInProgress: async () =>
      WriteContractComplete({
        exit: await Effect.runPromiseExit(executeContract(signingClient, senderAddress, contractAddress, msg, funds))
      }),
    WriteContractComplete: () => ts
  })

export const hasFailedExit = (state: TransactionSubmissionCosmos) =>
  "exit" in state && state.exit._tag === "Failure"

export const isComplete = (state: TransactionSubmissionCosmos) =>
  state._tag === "WriteContractComplete" && state.exit._tag === "Success"
