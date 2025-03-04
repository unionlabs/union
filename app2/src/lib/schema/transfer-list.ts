import { Schema } from "effect"
import { ChainReference } from "$lib/schema/chain"
import { Hex } from "$lib/schema/hex"
import { SortOrder } from "$lib/schema/sort-order"
import { PacketHash } from "$lib/schema/packet"
import { TokenRawDenom, TokenRawAmount } from "$lib/schema/token"

export class TransferListItem extends Schema.Class<TransferListItem>("TransferListItem")({
  source_chain: ChainReference,
  destination_chain: ChainReference,
  sender_canonical: Hex,
  receiver_canonical: Hex,
  packet_send_timestamp: Schema.DateTimeUtc,
  packet_recv_timestamp: Schema.OptionFromNullOr(Schema.DateTimeUtc),
  packet_hash: PacketHash,
  sort_order: SortOrder,
  base_token: TokenRawDenom,
  base_amount: TokenRawAmount,
  quote_token: TokenRawDenom,
  quote_amount: TokenRawAmount
}) {}

export const TransferList = Schema.Array(TransferListItem)

export const AggregateCount = Schema.Struct({
  count: Schema.Number
})
export const TransferCount = Schema.Struct({
  aggregate: AggregateCount
})
