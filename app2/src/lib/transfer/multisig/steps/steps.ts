import type { TransferContext } from "$lib/transfer/shared/services/filling/create-context.ts"
import { Data } from "effect"
import type { ExtractTag } from "effect/Types"

/**
 * Defines the different steps in a transfer process
 */
export type Steps = Data.TaggedEnum<{
  Filling: {}
  CheckMessage: {
    context: TransferContext
  }
}>

export type Filling = ExtractTag<Steps, "Filling">
export type CheckMessage = ExtractTag<Steps, "CheckMessage">

// Create constructors for the steps
export const { $match: match, $is: is, Filling, CheckMessage } = Data.taggedEnum<Steps>()

/**
 * Get a human-readable description for a transfer step
 */
export const description = match({
  Filling: () => "Configure your transfer details",
  CheckMessage: () => "Check your message",
})
