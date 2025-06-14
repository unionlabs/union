import type { Intent } from "$lib/transfer/shared/services/filling/create-context.ts"
import type {
  AddressCanonicalBytes,
  Chain,
  TokenRawAmount,
  TokenRawDenom,
} from "@unionlabs/sdk/schema"
import type { Instruction } from "@unionlabs/sdk/ucs03/instruction.ts"
import { Data, Option } from "effect"
import type { ExtractTag } from "effect/Types"

/**
 * Defines the different steps in a transfer process
 */
export type Steps = Data.TaggedEnum<{
  Filling: {}
  CheckReceiver: {
    readonly receiver: AddressCanonicalBytes
    readonly destinationChain: Chain
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
    readonly native: Option.Option<{
      baseToken: string
      amount: bigint
    }>
  }
  WaitForIndex: {
    readonly intent: Intent
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
  WaitForIndex,
} = Data.taggedEnum<Steps>()

/**
 * Get a human-readable description for a transfer step
 */
export const description = match({
  Filling: () => "Configure your transfer details",
  CheckReceiver: () => "Ensure correct receiver",
  ApprovalRequired: () => "Approve token spending",
  SubmitInstruction: () => "Submit transfer to blockchain",
  WaitForIndex: () => "Waiting for indexer",
})
