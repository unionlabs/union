import { Data } from "effect"
import type { Instruction } from "@unionlabs/sdk/ucs03"
import type { AddressCosmosDisplay, AddressEvmCanonical, Chain, Channel, TokenRawAmount, TokenRawDenom } from "@unionlabs/sdk/schema"
import type { ExtractTag } from "effect/Types"

/**
 * Defines the different steps in a transfer process
 */
export type TransferStep = Data.TaggedEnum<{
  Filling: {}
  ApprovalRequired: {
    readonly spender: AddressCosmosDisplay | AddressEvmCanonical
    readonly sourceChain: Chain
    readonly token: TokenRawDenom
    readonly requiredAmount: TokenRawAmount
    readonly currentAllowance: TokenRawAmount
  }
  SubmitInstruction: {
    readonly ucs03Address: AddressCosmosDisplay | AddressEvmCanonical
    readonly channel: Channel
    readonly timeoutTimestamp: bigint
    readonly sourceChain: Chain
    readonly instruction: Instruction.Instruction
  }
  WaitForIndex: {}
}>

export type Filling = ExtractTag<TransferStep, "Filling">
export type ApprovalRequired = ExtractTag<TransferStep, "ApprovalRequired">
export type SubmitInstruction = ExtractTag<TransferStep, "SubmitInstruction">
export type WaitForIndex = ExtractTag<TransferStep, "WaitForIndex">

// Create constructors for the steps
export const {
  $match: match,
  $is: is,
  Filling,
  ApprovalRequired,
  SubmitInstruction,
  WaitForIndex
} = Data.taggedEnum<TransferStep>()

/**
 * Get a human-readable description for a transfer step
 */
export const description = match({
  Filling: () => "Configure your transfer details",
  ApprovalRequired: () => "Approve token spending",
  SubmitInstruction: () => "Submit transfer to blockchain",
  WaitForIndex: () => "Waiting for indexer"
})
