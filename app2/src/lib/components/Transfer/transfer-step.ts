import { Data } from "effect"
import type { Instruction } from "@unionlabs/sdk/ucs03"
import type { TokenRawDenom } from "$lib/schema/token"
import type { Hash } from "viem";
import type {StdFee} from "@unionlabs/client";

/**
 * Defines the different steps in a transfer process
 */
export type TransferStep = Data.TaggedEnum<{
  Filling: {}
  ApprovalRequired: {
    readonly token: TokenRawDenom
    readonly requiredAmount: bigint
    readonly currentAllowance: bigint
  }
  SubmitInstruction: {
    readonly instruction: Instruction
  }
  WaitForIndex: {}
}>

// Create constructors for the steps
export const { Filling, ApprovalRequired, SubmitInstruction, WaitForIndex } = Data.taggedEnum<TransferStep>()

/**
 * Get a human-readable description for a transfer step
 */
export function getStepDescription(step: TransferStep): string {
  if (step._tag === "Filling") {
    return "Configure your transfer details"
  }
  if (step._tag === "ApprovalRequired") {
    return "Approve token spending"
  }
  if (step._tag === "SubmitInstruction") {
    return "Submit transfer to blockchain"
  }
  if (step._tag === "WaitForIndex") {
    return "Waiting for indexer"
  }
  return "Transfer step"
}
