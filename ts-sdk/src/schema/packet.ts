import * as S from "effect/Schema"
import { SortOrder } from "./sort-order.js"
import { Hex } from "./hex.js"
import { TransactionHash } from "./transaction.js"
import { ChainId, UniversalChainId } from "./chain.js"
import { ChannelId, ChannelVersion } from "./channel.js"
import { ConnectionId } from "./connection.js"
import { ClientId } from "./client.js"
import { PortId } from "./port.js"
import { Height } from "./height.js"

export const PacketHash = S.String.pipe(S.pattern(/^0x[0-9a-f]{64}$/)).pipe(
  S.brand("PacketHash")
)
export type PacketHash = typeof PacketHash.Type

export class PacketListItem extends S.Class<PacketListItem>("PacketListItem")({
  packet_hash: PacketHash,
  channel_version: ChannelVersion,
  destination_chain_id: ChainId,
  destination_channel_id: ChannelId,
  destination_universal_chain_id: UniversalChainId,
  source_channel_id: ChannelId,
  source_universal_chain_id: UniversalChainId,
  packet_send_timestamp: S.DateTimeUtc,
  packet_recv_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  packet_ack_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  sort_order: SortOrder,
  status: S.String
}) {}

export const PacketList = S.Array(PacketListItem)
export type PacketList = typeof PacketList.Type

export class PacketDetails extends S.Class<PacketDetails>("PacketDetails")({
  packet_hash: PacketHash,
  channel_version: ChannelVersion,
  data: Hex,
  destination_chain_id: ChainId,
  destination_channel_id: ChannelId,
  destination_client_id: ClientId,
  destination_connection_id: ConnectionId,
  destination_port_id: PortId,
  destination_universal_chain_id: UniversalChainId,
  packet_ack_block_hash: S.OptionFromNullOr(Hex),
  packet_ack_height: S.OptionFromNullOr(Height),
  packet_ack_maker: S.OptionFromNullOr(Hex),
  packet_ack_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  packet_ack_transaction_hash: S.OptionFromNullOr(TransactionHash),
  packet_recv_block_hash: S.OptionFromNullOr(Hex),
  packet_recv_height: S.OptionFromNullOr(Height),
  packet_recv_maker: S.OptionFromNullOr(Hex),
  packet_recv_maker_msg: S.OptionFromNullOr(S.String),
  packet_recv_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  packet_recv_transaction_hash: S.OptionFromNullOr(TransactionHash),
  packet_send_block_hash: S.OptionFromNullOr(Hex),
  packet_send_height: S.OptionFromNullOr(Height),
  packet_send_timestamp: S.DateTimeUtc,
  packet_send_transaction_hash: S.OptionFromNullOr(TransactionHash),
  sort_order: SortOrder,
  source_channel_id: ChannelId,
  source_client_id: ClientId,
  source_connection_id: ConnectionId,
  source_port_id: PortId,
  source_universal_chain_id: UniversalChainId,
  status: S.String,
  timeout_height: Height,
  timeout_timestamp: S.String,
  write_ack_block_hash: S.OptionFromNullOr(Hex),
  write_ack_height: S.OptionFromNullOr(Height),
  write_ack_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  write_ack_transaction_hash: S.OptionFromNullOr(TransactionHash),
  decoded: S.OptionFromNullOr(S.Any),
  decoded_flattened: S.OptionFromNullOr(S.Array(S.Any)),
  acknowledgement: S.OptionFromNullOr(S.Any)
}) {}

export const AggregateCount = S.Struct({
  count: S.Number
})
export type AggregateCount = typeof AggregateCount.Type

export const PacketCount = S.Struct({
  aggregate: AggregateCount
})
export type PacketCount = typeof PacketCount.Type
