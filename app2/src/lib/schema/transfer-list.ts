import { Schema } from "effect"
import { ChainId } from "$lib/schema/chain"
import { Hex } from "$lib/schema/hex"

export class TransferListItem extends Schema.Class<TransferListItem>("TransferListItem")({
  source_chain_id: ChainId,
  destination_chain_id: ChainId,
  sender_normalized: Hex,
  receiver_normalized: Hex
}) {}

export const TransferList = Schema.Array(TransferListItem)
