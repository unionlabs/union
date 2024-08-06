import { graphql } from "gql.tada"

export const ForwardFragment = graphql(/* GraphQL */ `
  fragment ForwardFragment on v0_transfers {
      forwards {
        port
        channel
        receiver
        chain { chain_id }
      }
  }
`)
