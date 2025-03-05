import { graphql } from "gql.tada"

export const transferListItemFragment = graphql(`
    fragment TransferListItem on v2_transfer_type {
        source_chain {
            chain_id
            universal_chain_id
        }
        destination_chain {
            chain_id
            universal_chain_id
        }
        sender_canonical
        receiver_canonical
        transfer_send_timestamp
        transfer_send_transaction_hash
        transfer_recv_timestamp
        packet_hash
        base_token
        base_amount
        quote_token
        quote_amount
        sort_order
    }
`)
