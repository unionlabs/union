import { Schema } from "effect"
import { ChainId } from "$lib/schema/chain"
import { AddressNormalized } from "$lib/schema/address"

export class TransferListItem extends Schema.Class<TransferListItem>("TransferListItem")({
  source_chain_id: ChainId,
  destination_chain_id: ChainId,
  sender_normalized: AddressNormalized,
  receiver_normalized: AddressNormalized
}) {}
