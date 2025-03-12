import { Schema } from "effect"
import { PortId } from "$lib/schema/port.ts"
import { ChainId } from "$lib/schema/chain.ts"
import { ConnectionId } from "$lib/schema/connection.ts"

export const ChannelId = Schema.Int.pipe(Schema.brand("ChannelId"))

export const ChannelVersion = Schema.String.pipe(Schema.brand("ChannelVersion"))

export class Channel extends Schema.Class<Channel>("Channel")({
  source_port_id: PortId,
  source_chain_id: ChainId,
  source_channel_id: ChannelId,
  source_connection_id: ConnectionId,
  destination_port_id: PortId,
  destination_chain_id: ChainId,
  destination_channel_id: ChannelId,
  destination_connection_id: ConnectionId
}) {}

export const Channels = Schema.Array(Channel)
