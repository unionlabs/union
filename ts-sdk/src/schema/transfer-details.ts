import * as S from "effect/Schema"
import * as Ucs05 from "../Ucs05.js"
import { AddressCanonicalBytes } from "./address.js"
import { ChainReference } from "./chain.js"
import { PacketTrace } from "./packet-trace.js"
import { TokenRawAmount, TokenRawDenom } from "./token.js"
import { TransactionHash } from "./transaction.js"

export class TransferDetails extends S.Class<TransferDetails>("TransferDetails")({
  sender_canonical: AddressCanonicalBytes,
  sender_display: Ucs05.AnyDisplayFromString,
  source_chain: ChainReference,
  transfer_send_transaction_hash: S.String,
  receiver_canonical: AddressCanonicalBytes,
  receiver_display: Ucs05.AnyDisplayFromString,
  destination_chain: ChainReference,
  transfer_send_timestamp: S.DateTimeUtc,
  transfer_recv_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  transfer_timeout_transaction_hash: S.OptionFromNullOr(TransactionHash),
  base_token: TokenRawDenom,
  base_amount: TokenRawAmount,
  quote_amount: TokenRawAmount,
  quote_token: TokenRawDenom,
  success: S.OptionFromNullOr(S.Boolean),
  traces: S.Array(PacketTrace),
}) {}
