import { graphql } from "../index.ts"

export const transferListDataFragment = graphql(`
    fragment TransferListData on v1_ibc_union_fungible_asset_orders {
        sender
        sender_normalized
        source_chain_id
        packet_send_timestamp
        packet_send_transaction_hash
        receiver
        receiver_normalized
        destination_chain_id
        packet_recv_timestamp
        base_token
        base_amount
        quote_token
        quote_amount
    }
`)
