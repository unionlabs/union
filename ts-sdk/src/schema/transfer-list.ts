import * as S from "effect/Schema"
import { AggregateCount } from "./aggregate-count.js"
import { ChainReference } from "./chain.js"
import { Hex } from "./hex.js"
import { PacketHash } from "./packet.js"
import { SortOrder } from "./sort-order.js"
import { TokenRawAmount, TokenRawDenom } from "./token.js"

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
  quote_amount: TokenRawAmount,
}) {}

export const TransferList = S.Array(TransferListItem)
export type TransferList = typeof TransferList.Type

export const TransferCount = S.Struct({
  aggregate: AggregateCount,
})
export type TransferCount = typeof TransferCount.Type
