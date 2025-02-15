import { graphql } from "gql.tada"

export const clientHeightsQuery = graphql(/* GraphQL */ `
     query MyQuery {
        v1_ibc_union_client_heights_max {
            client_chain {
                chain_id
            }
            counterparty_chain {
                chain_id
            }
            client_id
            max_counterparty_height
        }
        v1_ibc_union_chains {
            index_status {
                chain_id
                height
            }
        }
    }

  `)
