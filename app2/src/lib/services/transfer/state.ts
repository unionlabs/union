import { Data, type Exit, type Effect } from "effect"
import type { Hash } from "viem"
import type { submitTransfer, waitForReceipt } from "./transactions.ts"
import type { switchChain } from "./chain.ts"

type EffectToExit<T> = T extends Effect.Effect<infer A, infer E, any> ? Exit.Exit<A, E> : never

export type SwitchChainState = Data.TaggedEnum<{
  InProgress: {}
  Complete: { exit: EffectToExit<ReturnType<typeof switchChain>> }
}>
export const SwitchChainState = Data.taggedEnum<SwitchChainState>()

export type ApprovalSubmitState = Data.TaggedEnum<{
  InProgress: {}
  Complete: {
    exit: EffectToExit<ReturnType<typeof switchChain>> // TODO: change function
  }
}>
export const ApprovalSubmitState = Data.taggedEnum<ApprovalSubmitState>()

export type ApprovalReceiptState = Data.TaggedEnum<{
  InProgress: { readonly hash: Hash }
  Complete: { exit: EffectToExit<typeof waitForReceipt> }
}>
export const ApprovalReceiptState = Data.taggedEnum<ApprovalReceiptState>()

export type TransferSubmitState = Data.TaggedEnum<{
  InProgress: {}
  Complete: { exit: EffectToExit<ReturnType<typeof submitTransfer>> }
}>
export const TransferSubmitState = Data.taggedEnum<TransferSubmitState>()

export type TransferReceiptState = Data.TaggedEnum<{
  InProgress: { readonly hash: Hash }
  Complete: { exit: EffectToExit<ReturnType<typeof waitForReceipt>> }
}>
export const TransferReceiptState = Data.taggedEnum<TransferReceiptState>()

export type TransferSubmission2 = Data.TaggedEnum<{
  Pending: {}
  SwitchChain: { state: SwitchChainState }
  ApprovalSubmit: { state: ApprovalSubmitState }
  ApprovalReceipt: { state: ApprovalReceiptState }
  TransferSubmit: { state: TransferSubmitState }
  TransferReceipt: { state: TransferReceiptState }
}>

export const TransferSubmission2 = Data.taggedEnum<TransferSubmission2>()

export type TransferSubmission = Data.TaggedEnum<{
  Pending: {}
  SwitchChain: { state: SwitchChainState }
  TransferSubmit: { state: TransferSubmitState }
  TransferReceipt: { state: TransferReceiptState }
}>

export const TransferSubmission = Data.taggedEnum<TransferSubmission>()

type StateWithExit =
  | { _tag: "SwitchChain"; state: SwitchChainState }
  | { _tag: "TransferSubmit"; state: TransferSubmitState }
  | { _tag: "TransferReceipt"; state: TransferReceiptState }
  | { _tag: "ApprovalSubmit"; state: ApprovalSubmitState }
  | { _tag: "ApprovalReceipt"; state: ApprovalReceiptState }

export function hasFailedExit(state: StateWithExit | { _tag: "Pending" }): boolean {
  if (state._tag === "Pending") return false
  return state.state._tag === "Complete" && state.state.exit._tag === "Failure"
}

export function isComplete(state: StateWithExit | { _tag: "Pending" }): boolean {
  return (
    state._tag === "TransferReceipt" &&
    state.state._tag === "Complete" &&
    state.state.exit._tag === "Success"
  )
}
