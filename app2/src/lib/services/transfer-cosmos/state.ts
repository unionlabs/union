import { Data, type Exit, type Effect } from "effect"
import type { submitTransfer } from "./transactions.ts"
import type { switchChain } from "./chain.ts"
import type { approveTransfer } from "$lib/services/transfer-cosmos/approval.ts"

type EffectToExit<T> = T extends Effect.Effect<infer A, infer E, any> ? Exit.Exit<A, E> : never

export type SwitchChainState = Data.TaggedEnum<{
  InProgress: {}
  Complete: { exit: EffectToExit<ReturnType<typeof switchChain>> }
}>
export const SwitchChainState = Data.taggedEnum<SwitchChainState>()

export type ApprovalSubmitState = Data.TaggedEnum<{
  InProgress: {}
  Complete: {
    exit: EffectToExit<ReturnType<typeof approveTransfer>>
  }
}>
export const ApprovalSubmitState = Data.taggedEnum<ApprovalSubmitState>()

export type TransferSubmitState = Data.TaggedEnum<{
  InProgress: {}
  Complete: { exit: EffectToExit<ReturnType<typeof submitTransfer>> }
}>
export const TransferSubmitState = Data.taggedEnum<TransferSubmitState>()

export type TransferSubmission = Data.TaggedEnum<{
  Filling: {}
  SwitchChain: { state: SwitchChainState }
  ApprovalSubmit: { state: ApprovalSubmitState }
  TransferSubmit: { state: TransferSubmitState }
}>

export const TransferSubmission = Data.taggedEnum<TransferSubmission>()

type StateWithExit =
  | { _tag: "SwitchChain"; state: SwitchChainState }
  | { _tag: "ApprovalSubmit"; state: ApprovalSubmitState }
  | { _tag: "TransferSubmit"; state: TransferSubmitState }

export function hasFailedExit(state: StateWithExit | { _tag: "Filling" }): boolean {
  if (state._tag === "Filling") return false
  return state.state._tag === "Complete" && state.state.exit._tag === "Failure"
}

export function isComplete(state: StateWithExit | { _tag: "Filling" }): boolean {
  return (
    state._tag === "TransferSubmit" &&
    state.state._tag === "Complete" &&
    state.state.exit._tag === "Success"
  )
}
