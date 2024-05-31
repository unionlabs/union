import { graphql } from "gql.tada"

export const packetsQuery = graphql(/* GraphQL */ `
query PacketsQuery($limit: Int = 100) {
  v0_packets(limit: $limit, order_by: {destination_time: desc_nulls_last, source_time: desc_nulls_last}) {
    from_chain_id
    from_channel_id
    source_port
    source_block_hash
    source_time
    to_chain_id
    to_channel_id
    to_port_id
    destination_block_hash
    destination_time
    source_data
    status
  }
}
`)
