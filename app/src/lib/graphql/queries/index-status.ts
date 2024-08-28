import {graphql} from "gql.tada"

export const indexStatusQuery = graphql(/* GraphQL */ `
    query IndexStatusQuery {
        chains: v1_chains(where: {enabled:{_eq:true}}) {
            chain_id
        }
        statuses: v1_index_status(order_by: {status: desc}) {
            chain_id
            display_name
            height
            status
            timestamp
            tip_age_seconds
        }
    }
`)
