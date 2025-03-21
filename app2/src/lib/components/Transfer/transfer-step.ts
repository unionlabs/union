import { Data } from "effect"
import type { Instruction } from "@unionlabs/sdk/ucs03"
import type { TokenRawDenom } from "$lib/schema/token"

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
}>

// Create constructors for the steps
export const { Filling, ApprovalRequired, SubmitInstruction } = Data.taggedEnum<TransferStep>()
