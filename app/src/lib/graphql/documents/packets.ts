import { graphql } from "gql.tada"

export const packetsQuery = graphql(/* GraphQL */ `
query PacketsQuery($limit: Int = 100) {
  v0_packets(limit: $limit, order_by: {destination_time: desc_nulls_last, source_time: desc_nulls_last}) {
    source_chain_id
    source_port
    source_block_hash
    source_time
    destination_chain_id
    destination_block_hash
    destination_time
    source_data
    status
  }
}
`)
