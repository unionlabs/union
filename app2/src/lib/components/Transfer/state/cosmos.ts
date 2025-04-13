import { Data, Duration, Effect, type Exit, Schedule } from "effect"
import { switchChain } from "$lib/services/transfer-ucs03-cosmos"
import { executeContract } from "@unionlabs/sdk/cosmos"
import type { Chain } from "@unionlabs/sdk/schema"
import type { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate"

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
  WriteContractComplete
} = TransactionSubmissionCosmos

export const nextStateCosmos = async (
  ts: TransactionSubmissionCosmos,
  chain: Chain,
  signingClient: SigningCosmWasmClient,
  senderAddress: string,
  contractAddress: string,
  msg: Record<string, unknown>,
  funds?: ReadonlyArray<{ denom: string; amount: string }>
): Promise<TransactionSubmissionCosmos> =>
  TransactionSubmissionCosmos.$match(ts, {
    Filling: () => SwitchChainInProgress(),
    SwitchChainInProgress: async () => {
      const switchResult = await Effect.runPromiseExit(switchChain(chain))
      return SwitchChainComplete({
        exit: switchResult
      })
    },
    SwitchChainComplete: ({ exit }) => {
      if (exit._tag === "Failure") {
        console.error("[SwitchChainComplete] Chain switch failed with error:", exit.cause)
        console.log("[SwitchChainComplete] → Retrying SwitchChainInProgress")
        return SwitchChainInProgress()
      }
      console.log(
        "[SwitchChainComplete] Chain switch successful. → Moving to ExecuteContractInProgress"
      )
      return WriteContractInProgress()
    },
    WriteContractInProgress: async () => {
      const retryableExecute = executeContract(
        signingClient,
        senderAddress,
        contractAddress,
        msg,
        funds
      ).pipe(
        Effect.retry(
          Schedule.exponential(Duration.millis(100)).pipe(
            Schedule.whileInput(
              (err) =>
                err instanceof Error &&
                err.message.includes("429")
            ),
            Schedule.intersect(Schedule.recurs(5))
          )
        )
      )

      return WriteContractComplete({
        exit: await Effect.runPromiseExit(retryableExecute)
      })
    },


    WriteContractComplete: ({ exit }) => {
      if (exit._tag === "Failure") {
        console.error("[ExecuteContractComplete] Contract execution failed with error:", exit.cause)
        console.log("[ExecuteContractComplete] → Retrying ExecuteContractInProgress")
        return WriteContractInProgress()
      }
      console.log("ExecuteContractComplete] Contract execution successful. Transaction complete!")
      return ts
    }
  })

export const hasFailedExit = (state: TransactionSubmissionCosmos) =>
  "exit" in state && state.exit._tag === "Failure"

export const isComplete = (state: TransactionSubmissionCosmos): string | false => {
  if (state._tag === "WriteContractComplete" && state.exit._tag === "Success") {
    return state.exit.value.transactionHash
  }
  return false
}
