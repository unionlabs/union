import { graphql } from "gql.tada"

export const transferListItemFragment = graphql(`
    fragment TransferListItem on v2_transfer_list_item_type {
        source_chain {
            universal_chain_id
            chain_id
        }
        destination_chain {
            universal_chain_id
            chain_id
        }
        sender_canonical
        receiver_canonical
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
