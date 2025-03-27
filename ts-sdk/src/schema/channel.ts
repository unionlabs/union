import * as S from "effect/Schema"
import { PortId } from "./port.js"
import { ChainId } from "./chain.js"
import { ConnectionId } from "./connection.js"

export const ChannelId = S.Int.pipe(S.brand("ChannelId"))
export type ChannelId = typeof ChannelId.Type

export const ChannelVersion = S.String.pipe(S.brand("ChannelVersion"))
export type ChannelVersion = typeof ChannelVersion.Type

export class Channel extends S.Class<Channel>("Channel")({
  source_port_id: PortId,
  source_chain_id: ChainId,
  source_channel_id: ChannelId,
  source_connection_id: ConnectionId,
  destination_port_id: PortId,
  destination_chain_id: ChainId,
  destination_channel_id: ChannelId,
  destination_connection_id: ConnectionId
}) {}

export const Channels = S.Array(Channel)
export type Channels = typeof Channels.Type
