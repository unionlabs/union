import { graphql } from "gql.tada"

export const transferListItemFragment = graphql(`
    fragment TransferListItem on v1_ibc_union_fungible_asset_orders {
        source_chain_id
        destination_chain_id
        sender_normalized
        receiver_normalized
        packet_send_timestamp
        packet_send_transaction_hash
        packet_recv_timestamp
        packet_hash
        base_token
        base_amount
        quote_token
        quote_amount
        sort_order
    }
`)
