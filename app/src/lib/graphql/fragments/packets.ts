import { graphql } from "gql.tada"

export const packetListDataFragment = graphql(/* GraphQL */ `
  fragment PacketListData on v1_packets {
      source_chain_id
      source_connection_id
      source_channel_id
      source_port_id
      source_block_hash
      source_timestamp
      source_sequence
      destination_chain_id
      destination_connection_id
      destination_channel_id
      destination_port_id
      destination_block_hash
      destination_timestamp
      destination_sequence
  }
`)

export const packetDetailsFragment = graphql(/* GraphQL */ `
    fragment PacketDetails on v1_packets {
        source_chain_id,
        source_block_hash,
        source_height,
        source_timestamp,
        source_transaction_hash,
        source_transaction_index,
        source_sequence,
        source_port_id,
        source_channel_id,
        source_timeout_timestamp,
        source_event_json,
        source_packet_data,
        source_chain_id,
        source_connection_id,
        source_channel_id,
        source_port_id,
        destination_chain_id,
        destination_connection_id,
        destination_channel_id,
        destination_port_id,
        destination_chain_id,
        destination_block_hash,
        destination_height,
        destination_timestamp,
        destination_transaction_hash,
        destination_transaction_index,
        destination_sequence,
        destination_port_id,
        destination_channel_id,
        destination_timeout_timestamp,
        destination_event_json,
        destination_packet_data
    }
`)
