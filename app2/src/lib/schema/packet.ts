import { Schema } from "effect"
import { SortOrder } from "$lib/schema/sort-order"
import { Hex } from "$lib/schema/hex"
import { ChainId, UniversalChainId } from "$lib/schema/chain"
import { ChannelId } from "$lib/schema/channel"
import { ConnectionId } from "$lib/schema/connection"
import { ClientId } from "$lib/schema/client"
import { PortId } from "$lib/schema/port"

export const PacketHash = Schema.String.pipe(Schema.pattern(/^0x[0-9a-f]{64}$/)).pipe(
  Schema.brand("PacketHash")
)

export type PacketHash = typeof PacketHash.Type

export class PacketListItem extends Schema.Class<PacketListItem>("PacketListItem")({
  packet_hash: PacketHash,
  channel_version: Schema.String,
  destination_chain_id: ChainId,
  destination_channel_id: ChannelId,
  destination_universal_chain_id: UniversalChainId,
  source_channel_id: ChannelId,
  source_universal_chain_id: UniversalChainId,
  packet_send_timestamp: Schema.DateTimeUtc,
  packet_recv_timestamp: Schema.OptionFromNullOr(Schema.DateTimeUtc),
  packet_ack_timestamp: Schema.OptionFromNullOr(Schema.DateTimeUtc),
  sort_order: SortOrder,
  status: Schema.String
}) {}

export const PacketList = Schema.Array(PacketListItem)


export class PacketDetails extends Schema.Class<PacketDetails>("PacketDetails")({
  packet_hash: PacketHash,
  channel_version: Schema.String,
  data: Schema.Any,
  destination_chain_id: ChainId,
  destination_channel_id: ChannelId,
  destination_client_id: ClientId,
  destination_connection_id: ConnectionId,
  destination_port_id: PortId,
  destination_universal_chain_id: UniversalChainId,
  packet_ack_block_hash: Schema.OptionFromNullOr(Hex),
  packet_ack_height: Schema.OptionFromNullOr(Schema.Number),
  packet_ack_maker: Schema.OptionFromNullOr(Schema.String),
  packet_ack_timestamp: Schema.OptionFromNullOr(Schema.DateTimeUtc),
  packet_ack_transaction_hash: Schema.OptionFromNullOr(Hex),
  packet_recv_block_hash: Schema.OptionFromNullOr(Hex),
  packet_recv_height: Schema.OptionFromNullOr(Schema.Number),
  packet_recv_maker: Schema.OptionFromNullOr(Schema.String),
  packet_recv_maker_msg: Schema.OptionFromNullOr(Schema.String),
  packet_recv_timestamp: Schema.OptionFromNullOr(Schema.DateTimeUtc),
  packet_recv_transaction_hash: Schema.OptionFromNullOr(Hex),
  packet_send_block_hash: Schema.OptionFromNullOr(Hex),
  packet_send_height: Schema.OptionFromNullOr(Schema.Number),
  packet_send_timestamp: Schema.DateTimeUtc,
  packet_send_transaction_hash: Schema.OptionFromNullOr(Hex),
  sort_order: SortOrder,
  source_channel_id: ChannelId,
  source_client_id: ClientId,
  source_connection_id: ConnectionId,
  source_port_id: PortId,
  source_universal_chain_id: UniversalChainId,
  status: Schema.String,
  timeout_height: Schema.String,
  timeout_timestamp: Schema.String,
  write_ack_block_hash: Schema.OptionFromNullOr(Hex),
  write_ack_height: Schema.OptionFromNullOr(Schema.Number),
  write_ack_timestamp: Schema.OptionFromNullOr(Schema.DateTimeUtc),
  write_ack_transaction_hash: Schema.OptionFromNullOr(Hex),
  decoded: Schema.OptionFromNullOr(Schema.Any),
  decoded_flattened: Schema.OptionFromNullOr(Schema.Array(Schema.Any)),
  acknowledgement: Schema.OptionFromNullOr(Schema.Any)
}) {}

export const AggregateCount = Schema.Struct({
  count: Schema.Number
})

export const PacketCount = Schema.Struct({
  aggregate: AggregateCount
})
