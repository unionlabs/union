import { Schema } from "effect"
import * as S from "effect/Schema"
import { UniversalChainId } from "./chain.js"
import { ClientId } from "./client.js"
import { ConnectionId } from "./connection.js"
import { Fees } from "./fee.js"
import { PortId } from "./port.js"

export const ChannelId = S.Int.pipe(S.brand("ChannelId"))
export type ChannelId = typeof ChannelId.Type

export const ChannelVersion = S.String.pipe(S.brand("ChannelVersion"))
export type ChannelVersion = typeof ChannelVersion.Type

export class Channel extends Schema.Class<Channel>("Channel")({
  destination_channel_id: ChannelId,
  destination_client_id: ClientId,
  destination_connection_id: ConnectionId,
  destination_port_id: PortId,
  destination_universal_chain_id: UniversalChainId,
  fees: Fees,
  source_channel_id: ChannelId,
  source_client_id: ClientId,
  source_connection_id: ConnectionId,
  source_port_id: PortId,
  source_universal_chain_id: UniversalChainId,
}) {}

export const Channels = S.Array(Channel)
export type Channels = typeof Channels.Type
