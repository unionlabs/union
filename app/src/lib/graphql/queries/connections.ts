import { graphql } from "gql.tada"

export const connectionsQuery = graphql(/* GraphQL */ `
    query ConnectionsQuery($limit: Int = 100) @cached(ttl: 30) {
        v1_ibc_union_channel_recommendations(
            order_by: [
                {source_chain_id: asc},
                {source_client_id: asc},
                {source_connection_id: asc},
                {destination_chain_id: asc},
                {destination_client_id: asc},
                {destination_connection_id: asc}
            ],
            limit: $limit
        ) {
            source_chain_id
            destination_chain_id
            source_client_id
            source_connection_id
            source_chain {
                enabled
                display_name
            }
            destination_client_id
            destination_connection_id
            destination_chain {
                enabled
                display_name
            }
            status
        }
    }
`)
