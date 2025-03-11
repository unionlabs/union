import { Schema } from "effect"

export const ChannelId = Schema.Int.pipe(Schema.brand("ChannelId"))

export const ChannelVersion = Schema.String.pipe(Schema.brand("ChannelVersion"))
