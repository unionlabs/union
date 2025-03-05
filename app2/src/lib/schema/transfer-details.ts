import { Schema } from "effect"
import { ChainReference } from "$lib/schema/chain"
import { Hex } from "$lib/schema/hex"
import { PacketTrace } from "$lib/schema/packet-trace"
import { TokenRawDenom, TokenRawAmount } from "$lib/schema/token"
import { AddressCanonicalBytes } from "./address"

export class TransferDetails extends Schema.Class<TransferDetails>("TransferDetails")({
  sender_canonical: AddressCanonicalBytes,
  source_chain: ChainReference,
  packet_send_transaction_hash: Schema.String,
  receiver_canonical: AddressCanonicalBytes,
  destination_chain: ChainReference,
  packet_send_timestamp: Schema.DateTimeUtc,
  packet_recv_timestamp: Schema.OptionFromNullOr(Schema.DateTimeUtc),
  base_token: TokenRawDenom,
  base_amount: TokenRawAmount,
  quote_amount: TokenRawAmount,
  quote_token: TokenRawDenom,
  traces: Schema.Array(PacketTrace)
}) {}
