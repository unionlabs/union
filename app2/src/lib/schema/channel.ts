import { Schema } from "effect"

export const ChannelId = Schema.Int.pipe(
  Schema.nonNegative({ message: () => "sourceChannelId must be non-negative" }),
  Schema.brand("ChannelId")
)