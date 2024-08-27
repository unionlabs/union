import { graphql } from "gql.tada"

export const packetListDataFragment = graphql(/* GraphQL */ `
  fragment PacketListData on v0_packets {
    from_chain_id
    from_connection_id
    from_channel_id
    from_port_id
    source_block_hash
    source_time
    source_sequence
    to_chain_id
    to_connection_id
    to_channel_id
    to_port_id
    destination_block_hash
    destination_time
    destination_sequence
    status
  }
`)

export const packetDetailsFragment = graphql(/* GraphQL */ `
  fragment PacketDetails on v0_packets {
    source_chain_id,
    source_block_hash,
    source_height,
    source_time,
    source_transaction_hash,
    source_transaction_index,
    source_sequence,
    source_port,
    source_channel,
    source_timeout_timestamp,
    source_json,
    source_data,
    from_chain_id,
    from_connection_id,
    from_channel_id,
    from_port_id,
    to_chain_id,
    to_connection_id,
    to_channel_id,
    to_port_id,
    status,
    destination_chain_id,
    destination_block_hash,
    destination_height,
    destination_time,
    destination_transaction_hash,
    destination_transaction_index,
    destination_sequence,
    destination_port,
    destination_channel,
    destination_timeout_timestamp,
    destination_json,
    destination_data
  }
`)
