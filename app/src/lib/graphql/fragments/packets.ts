import { graphql } from "gql.tada"

export const packetListDataFragment = graphql(/* GraphQL */ `
  fragment PacketListData on v0_packets {
    from_chain_id
    from_connection_id
    from_channel_id
    from_port_id
    source_block_hash
    source_time
    to_chain_id
    to_connection_id
    to_channel_id
    to_port_id
    destination_block_hash
    destination_time
    source_data
    status
  }
`)
