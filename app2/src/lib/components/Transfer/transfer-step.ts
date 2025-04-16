import { Data } from "effect"
import type {
  AddressCanonicalBytes,
  Chain,
  TokenRawAmount,
  TokenRawDenom
} from "@unionlabs/sdk/schema"
import type { ExtractTag } from "effect/Types"
import type {TransferContext} from "$lib/components/Transfer/state/filling/create-intents.ts";
import type {Instruction} from "@unionlabs/sdk/ucs03/instruction.ts";

/**
 * Defines the different steps in a transfer process
 */
export type TransferStep = Data.TaggedEnum<{
  Filling: {}
  CheckReceiver: {
    receiver: AddressCanonicalBytes
    destinationChain: Chain
  }
  ApprovalRequired: {
    readonly token: TokenRawDenom
    readonly requiredAmount: TokenRawAmount
    readonly currentAllowance: TokenRawAmount
    readonly context: TransferContext
  }
  SubmitInstruction: {
    readonly instruction: Instruction
    readonly context: TransferContext
  }
  WaitForIndex: {
    context: TransferContext
  }

}>

export type Filling = ExtractTag<TransferStep, "Filling">
export type CheckReceiver = ExtractTag<TransferStep, "CheckReceiver">
export type ApprovalRequired = ExtractTag<TransferStep, "ApprovalRequired">
export type SubmitInstruction = ExtractTag<TransferStep, "SubmitInstruction">
export type WaitForIndex = ExtractTag<TransferStep, "WaitForIndex">

// Create constructors for the steps
export const {
  $match: match,
  $is: is,
  Filling,
  CheckReceiver,
  ApprovalRequired,
  SubmitInstruction,
  WaitForIndex
} = Data.taggedEnum<TransferStep>()

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
