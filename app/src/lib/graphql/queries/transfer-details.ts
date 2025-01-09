import { graphql } from "../index.ts"

export const ucs03OrdersBySourceHashQueryDocument = graphql(/*graphql*/ `
query Ucs03OrdersBySourceHash($source_transaction_hash: String!) @cached(ttl: 1) {
  v1_ibc_union_fungible_asset_order_view(limit: 10, where: {packet_send_transaction_hash: {_eq: $source_transaction_hash}}) {
    ack_fill_type
    ack_market_maker
    ack_tag
    acknowledgement
    acknowledgement_decoded
    base_amount
    base_token
    base_token_name
    base_token_path
    base_token_symbol
    channel_version
    data
    data_decoded
    destination_chain_id
    destination_channel_id
    destination_connection_id
    destination_port_id
    internal_destination_chain_id
    internal_source_chain_id
    packet_ack_block_hash
    packet_ack_event_index
    packet_ack_height
    packet_ack_maker
    packet_ack_timestamp
    packet_ack_transaction_event_index
    packet_ack_transaction_hash
    packet_ack_transaction_index
    packet_recv_block_hash
    packet_recv_event_index
    packet_recv_height
    packet_recv_maker
    packet_recv_maker_msg
    packet_recv_timestamp
    packet_recv_transaction_event_index
    packet_recv_transaction_hash
    packet_recv_transaction_index
    packet_send_block_hash
    packet_send_event_index
    packet_send_height
    packet_send_timestamp
    packet_send_transaction_event_index
    packet_send_transaction_hash
    packet_send_transaction_index
    quote_amount
    quote_token
    receiver
    sender
    source_chain_id
    source_channel_id
    source_connection_id
    source_port_id
    status
    timeout_height
    timeout_timestamp
    write_ack_block_hash
    write_ack_event_index
    write_ack_height
    write_ack_timestamp
    write_ack_transaction_event_index
    write_ack_transaction_hash
    write_ack_transaction_index
  }
}
`)

export const transfersBySourceHashBaseQueryDocument = graphql(/* GraphQL */ `
    query TransfersBySourceHashBase($source_transaction_hash: String!)
    @cached(ttl: 1) {
        v1_transfers(
            where: { source_transaction_hash: { _eq: $source_transaction_hash } }
        ) {
            sender
            normalized_sender
            source_chain_id
            source_connection_id
            source_channel_id
            source_sequence
            source_transaction_hash
            receiver
            normalized_receiver
            destination_chain_id
            destination_connection_id
            destination_channel_id
            destination_sequence
            tokens {
                denom
                amount
                asset {
                    denom
                    decimals
                    display_name
                    display_symbol
                }
            }
            source_timestamp
            destination_timestamp
            forwards {
                source_connection_id
                source_channel_id
                destination_connection_id
                destination_channel_id
                destination_chain_id
                source_channel_id
                receiver
            }
        }
    }
`)

export const transfersBySourceHashTracesAndHopsQueryDocument = graphql(/* GraphQL */ `
    query TransfersBySourceHashTracesAndHops($source_transaction_hash: String!)
    @cached(ttl: 1) {
      v1_transfers(
        where: { source_transaction_hash: { _eq: $source_transaction_hash } }
      ) {
        traces(order_by: { timestamp: asc }) {
          timestamp
          chain {
            chain_id
          }
          type
          transaction_hash
          height
        }
        hop {
          traces(order_by: { timestamp: asc }) {
            timestamp
            chain {
              chain_id
            }
            type
            transaction_hash
            height
          }
        }
      }
    }
  `)
