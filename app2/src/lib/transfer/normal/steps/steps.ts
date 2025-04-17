import { Data } from "effect"
import type {
  AddressCanonicalBytes,
  Chain,
  TokenRawAmount,
  TokenRawDenom
} from "@unionlabs/sdk/schema"
import type { ExtractTag } from "effect/Types"
import type { Intent } from "$lib/transfer/shared/services/filling/create-context.ts"
import type { Instruction } from "@unionlabs/sdk/ucs03/instruction.ts"

/**
 * Defines the different steps in a transfer process
 */
export type Steps = Data.TaggedEnum<{
  Filling: {}
  CheckReceiver: {
    receiver: AddressCanonicalBytes
    destinationChain: Chain
  }
  ApprovalRequired: {
    readonly token: TokenRawDenom
    readonly requiredAmount: TokenRawAmount
    readonly currentAllowance: TokenRawAmount
    readonly intent: Intent
  }
  SubmitInstruction: {
    readonly instruction: Instruction
    readonly intent: Intent
  }
  WaitForIndex: {
    intent: Intent
  }
}>

export type Filling = ExtractTag<Steps, "Filling">
export type CheckReceiver = ExtractTag<Steps, "CheckReceiver">
export type ApprovalRequired = ExtractTag<Steps, "ApprovalRequired">
export type SubmitInstruction = ExtractTag<Steps, "SubmitInstruction">
export type WaitForIndex = ExtractTag<Steps, "WaitForIndex">

// Create constructors for the steps
export const {
  $match: match,
  $is: is,
  Filling,
  CheckReceiver,
  ApprovalRequired,
  SubmitInstruction,
  WaitForIndex
} = Data.taggedEnum<Steps>()

/**
 * Get a human-readable description for a transfer step
 */
export const description = match({
  Filling: () => "Configure your transfer details",
  CheckReceiver: () => "Ensure correct receiver",
  ApprovalRequired: () => "Approve token spending",
  SubmitInstruction: () => "Submit transfer to blockchain",
  WaitForIndex: () => "Waiting for indexer"
})
