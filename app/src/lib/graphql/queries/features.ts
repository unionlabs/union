import { graphql } from "gql.tada"

export const enabledFeatures = graphql(/* GraphQL */ `
    query enabledFeaturesQuery($environment: String!) {
        v1_ibc_union_chains {
            chain_id
            features(where: {environment: {_eq: $environment}}) {
                channel_list
                connection_list
                environment
                index_status
                packet_list
                transfer_list
                transfer_submission
            }
        }
    }
`)
