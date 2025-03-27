import { Schema } from "effect"
import { PortId } from "$lib/schema/port.ts"
import {UniversalChainId} from "$lib/schema/chain.ts"
import { ConnectionId } from "$lib/schema/connection.ts"
import {ClientId} from "$lib/schema/client.ts";

export const ChannelId = Schema.Int.pipe(Schema.brand("ChannelId"))
export type ChannelId = typeof ChannelId.Type

export const ChannelVersion = Schema.String.pipe(Schema.brand("ChannelVersion"))

export class Channel extends Schema.Class<Channel>("Channel")({
  destination_channel_id: ChannelId,
  destination_client_id: ClientId,
  destination_connection_id: ConnectionId,
  destination_port_id: PortId,
  destination_universal_chain_id: UniversalChainId,
  source_channel_id: ChannelId,
  source_client_id: ClientId,
  source_connection_id: ConnectionId,
  source_port_id: PortId,
  source_universal_chain_id: UniversalChainId
}) {}

export const Channels = Schema.Array(Channel)
