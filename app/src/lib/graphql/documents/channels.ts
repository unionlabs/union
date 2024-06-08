import { graphql } from "gql.tada"

export const channelsQuery = graphql(/* GraphQL */ `query ChannelsQuery($limit: Int = 500) @cached(ttl: 30) {
  v0_channel_map(order_by: [
    {status: asc}, 
    {from_chain_id: asc}, 
    {from_connection_id: asc},
    {from_channel_id: asc},
    {from_port_id: asc},
    {to_chain_id: asc}, 
    {to_connection_id: asc}, 
    {to_channel_id: asc},
    {to_port_id: asc}
  ], limit: $limit) {
    from_chain_id
    from_connection_id
    from_channel_id
    from_port_id
    to_chain_id
    to_connection_id
		to_channel_id    
    to_port_id
    status
  }
}`)
