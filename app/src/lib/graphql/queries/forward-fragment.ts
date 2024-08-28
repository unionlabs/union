import { graphql } from "gql.tada"

export const ForwardFragment = graphql(/* GraphQL */ `
  fragment ForwardFragment on v1_transfers {
      forwards {
          source_port_id
          source_channel_id
          receiver
          source_chain { chain_id }
      }
  }
`)
