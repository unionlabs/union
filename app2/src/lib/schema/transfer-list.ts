import { Schema } from "effect"
import { ChainId } from "$lib/schema/chain"
import { Hex } from "$lib/schema/hex"
import { SortOrder } from "$lib/schema/sort-order"

export class TransferListItem extends Schema.Class<TransferListItem>("TransferListItem")({
  source_chain_id: ChainId,
  destination_chain_id: ChainId,
  sender_normalized: Hex,
  receiver_normalized: Hex,
  packet_send_timestamp: Schema.DateTimeUtc,
  packet_recv_timestamp: Schema.OptionFromNullOr(Schema.DateTimeUtc),
  sort_order: SortOrder
}) {}

export const TransferList = Schema.Array(TransferListItem)
