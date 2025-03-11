import { Schema } from "effect"

export const ChannelId = Schema.Int.pipe(Schema.brand("ChannelId"))

export const ChannelVersion = Schema.String.pipe(Schema.brand("ChannelVersion"))

export class Channel extends Schema.Class<Channel>("Channel")({
  source_port_id: Schema.String,
  source_chain_id: Schema.String,
  source_channel_id: Schema.Int,
  source_connection_id: Schema.Int,
  destination_port_id: Schema.String,
  destination_chain_id: Schema.String,
  destination_channel_id: Schema.Int,
  destination_connection_id: Schema.Int
}) {}

export const Channels = Schema.Array(Channel)
