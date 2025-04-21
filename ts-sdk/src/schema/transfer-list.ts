import * as S from "effect/Schema"
import { ChainReference } from "./chain.js"
import { Hex } from "./hex.js"
import { SortOrder } from "./sort-order.js"
import { PacketHash } from "./packet.js"
import { TokenRawDenom, TokenRawAmount } from "./token.js"
import { AggregateCount } from "./aggregate-count.js"

export class TransferListItem extends S.Class<TransferListItem>("TransferListItem")({
  source_chain: ChainReference,
  destination_chain: ChainReference,
  sender_canonical: Hex,
  receiver_canonical: Hex,
  transfer_send_timestamp: S.DateTimeUtc,
  transfer_recv_timestamp: S.OptionFromNullOr(S.DateTimeUtc),
  packet_hash: PacketHash,
  sort_order: SortOrder,
  base_token: TokenRawDenom,
  base_amount: TokenRawAmount,
  quote_token: TokenRawDenom,
  quote_amount: TokenRawAmount
}) {}

export const TransferList = S.Array(TransferListItem)
export type TransferList = typeof TransferList.Type

const TraceItem = S.Struct({
  type: S.String,
  transaction_hash: S.OptionFromNullOr(S.String),
  universal_chain_id: S.String,                // ← plain string, not ChainReference
  timestamp:         S.OptionFromNullOr(S.DateTimeUtc),
})

export class TransferListItemMissingAck extends S.Class<TransferListItemMissingAck>(
  "TransferListItemMissingAck"
)({
  source_chain:             ChainReference,
  destination_chain:        ChainReference,
  sender_canonical:         S.String, 
  receiver_canonical:       S.String,
  transfer_send_timestamp:  S.DateTimeUtc,
  transfer_recv_timestamp:  S.OptionFromNullOr(S.DateTimeUtc),
  packet_hash:              S.String,
  sort_order:               S.String,
  base_token:               S.String,
  base_amount:              S.String,
  quote_token:              S.String,
  quote_amount:             S.String,
  traces:                   S.Array(TraceItem),
}) {}

export const TransferListMissingAck = S.Array(TransferListItemMissingAck)
export type TransferListMissingAck = typeof TransferListMissingAck.Type


export const TransferCount = S.Struct({
  aggregate: AggregateCount
})
export type TransferCount = typeof TransferCount.Type
