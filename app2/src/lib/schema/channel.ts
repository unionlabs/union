import { Schema } from "effect"

export const ChannelId = Schema.Int.pipe(
  Schema.nonNegative({ message: () => "ChannelId must be non-negative" }),
  Schema.brand("ChannelId")
)
