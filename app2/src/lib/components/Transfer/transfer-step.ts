import { Data } from "effect"
import * as S from "effect/Schema"
import { Instruction } from "@unionlabs/sdk/ucs03"
import { TokenRawDenom } from "$lib/schema/token"

const Filling = S.TaggedStruct("Filling", {})
type Filling = typeof Filling.Type

const ApprovalRequired = S.TaggedStruct("ApprovalRequired", {
  token: TokenRawDenom,
  requiredAmount: S.BigIntFromSelf,
  currentAllowance: S.BigIntFromSelf
})
type ApprovalRequired = typeof ApprovalRequired.Type

const SubmitInstruction = S.TaggedStruct("SubmitInstruction", {
  instruction: Instruction.Schema
})

const Schema = S.Union(
  Filling,
  ApprovalRequired,
)
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
export const {
  $match: match,
  $is: is,
  SubmitInstruction
} = Data.taggedEnum<TransferStep>()

/**
 * Get a human-readable description for a transfer step
 */
export const getStepDescription = match({
  Filling: () => "Configure your transfer details.",
  ApprovalRequired: () => "Approve token spending.",
  SubmitInstruction: () => "Submit transfer to blockchain."
})