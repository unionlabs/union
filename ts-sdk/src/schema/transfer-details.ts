import * as S from "effect/Schema"
import { ChainReference } from "./chain.js"
import { PacketTrace } from "./packet-trace.js"
import { TokenRawDenom, TokenRawAmount } from "./token.js"
import { AddressCanonicalBytes } from "./address.js"

export class TransferDetails extends S.Class<TransferDetails>("TransferDetails")({
  sender_canonical: AddressCanonicalBytes,
  source_chain: ChainReference,
  transfer_send_transaction_hash: S.String,
  receiver_canonical: AddressCanonicalBytes,
  destination_chain: ChainReference,
  transfer_send_timestamp: S.DateTimeUtc,
  transfer_recv_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  base_token: TokenRawDenom,
  base_amount: TokenRawAmount,
  quote_amount: TokenRawAmount,
  quote_token: TokenRawDenom,
  success: S.OptionFromNullOr(S.Boolean),
  traces: S.Array(PacketTrace)
}) {}
