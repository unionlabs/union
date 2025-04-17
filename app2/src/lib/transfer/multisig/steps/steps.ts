import { Data } from "effect"
import type { ExtractTag } from "effect/Types"

/**
 * Defines the different steps in a transfer process
 */
export type Steps = Data.TaggedEnum<{
  Filling: {}
  CheckMessage: {
    message: any
  }
}>

export type Filling = ExtractTag<Steps, "Filling">
export type CheckReceiver = ExtractTag<Steps, "CheckMessage">

// Create constructors for the steps
export const {
  $match: match,
  $is: is,
  Filling,
  CheckMessage,
} = Data.taggedEnum<Steps>()

/**
 * Get a human-readable description for a transfer step
 */
export const description = match({
  Filling: () => "Configure your transfer details",
  CheckMessage: () => "Check your message",
})
