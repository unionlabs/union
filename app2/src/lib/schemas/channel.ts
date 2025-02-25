import { Schema } from "effect"

export const ChannelId = Schema.Int.pipe(Schema.brand("ChannelId"))
