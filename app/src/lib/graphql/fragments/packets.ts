import { graphql } from "gql.tada"

export const packetListDataFragment = graphql(/* GraphQL */ `
  fragment PacketListData on v1_ibc_union_packets {
      source_chain_id
      source_connection_id
      source_channel_id
      source_port_id
      packet_send_block_hash
      packet_send_timestamp
      destination_chain_id
      destination_connection_id
      destination_channel_id
      destination_port_id
      packet_recv_block_hash
      packet_recv_timestamp
      source_chain {
          chain_id
      }
      destination_chain {
          chain_id
      }
  }
`)

export const packetDetailsFragment = graphql(/* GraphQL */ `
    fragment PacketDetails on v1_ibc_union_packets {
        source_chain_id,
        packet_send_block_hash,
        packet_send_height,
        packet_send_timestamp,
        packet_send_transaction_hash,
        packet_send_transaction_index,
        source_port_id,
        source_channel_id,
        timeout_timestamp
        timeout_height
        data
        data_decoded
        source_chain_id,
        source_connection_id,
        source_channel_id,
        source_port_id,
        destination_chain_id,
        destination_connection_id,
        destination_channel_id,
        destination_port_id,
        destination_chain_id,
        packet_recv_block_hash,
        packet_recv_height,
        packet_recv_timestamp,
        packet_recv_transaction_hash,
        packet_recv_transaction_index,
        destination_port_id,
        destination_channel_id,
        source_chain {
            chain_id
        }
        destination_chain {
            chain_id
        }
    }
`)
