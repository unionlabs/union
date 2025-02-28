import { Schema } from "effect"
import { ChainId } from "$lib/schema/chain"
import { Hex } from "$lib/schema/hex"
import { PacketTrace } from "$lib/schema/packet-trace"

export class TransferDetails extends Schema.Class<TransferDetails>("TransferDetails")({
  sender_normalized: Hex,
  source_chain_id: ChainId,
  source_connection_id: Schema.Number,
  source_channel_id: Schema.Number,
  packet_send_transaction_hash: Schema.String,
  receiver_normalized: Hex,
  destination_chain_id: ChainId,
  destination_connection_id: Schema.Number,
  destination_channel_id: Schema.Number,
  packet_send_timestamp: Schema.DateTimeUtc,
  packet_recv_timestamp: Schema.OptionFromNullOr(Schema.DateTimeUtc),
  base_token: Schema.String,
  base_amount: Schema.String,
  quote_amount: Schema.String,
  quote_token: Schema.String,
  traces: Schema.Array(PacketTrace)
}) {}
