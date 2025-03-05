import { Schema } from "effect"
import { ChainReference } from "$lib/schema/chain"
import { PacketTrace } from "$lib/schema/packet-trace"
import { TokenRawDenom, TokenRawAmount } from "$lib/schema/token"
import { AddressCanonicalBytes } from "./address.ts"

export class TransferDetails extends Schema.Class<TransferDetails>("TransferDetails")({
  sender_canonical: AddressCanonicalBytes,
  source_chain: ChainReference,
  transfer_send_transaction_hash: Schema.String,
  receiver_canonical: AddressCanonicalBytes,
  destination_chain: ChainReference,
  transfer_send_timestamp: Schema.DateTimeUtc,
  transfer_recv_timestamp: Schema.OptionFromNullOr(Schema.DateTimeUtc),
  base_token: TokenRawDenom,
  base_amount: TokenRawAmount,
  quote_amount: TokenRawAmount,
  quote_token: TokenRawDenom,
  traces: Schema.Array(PacketTrace)
}) {}
