import { graphql } from "gql.tada"

export const connectionsQuery = graphql(/* GraphQL */ `
query ConnectionsQuery($limit: Int = 100) @cached(ttl: 30) {
  v0_connection_map(order_by: [
    {status: asc}, 
    {from_chain_id: asc}, 
    {from_client_id: asc}, 
    {from_connection_id: asc},
    {to_chain_id: asc}, 
    {to_client_id: asc}, 
    {to_connection_id: asc}
    ], limit: $limit) {
    from_chain_id
    to_chain_id
    from_client_id
    from_connection_id
    to_client_id
    to_connection_id
    status
  }
}
`)
