import { graphql } from "gql.tada"

export const ibcChannelMapQuery = graphql(/* graphql */ `
  query IBCChannelMapQuery($limit: Int = 500, $status: String = "CONFIRM") {
    data: v0_channel_map(limit: $limit, where: { status: { _eq: $status }}) {
      from_chain_id
      from_channel_id
      from_connection_id
      from_id
      from_port_id
      status
      to_chain_id
      to_channel_id
      to_connection_id
      to_id
      to_port_id
    }
  }
`)
