import { graphql } from "gql.tada"

export const explorerTransferFragment = graphql(/* GraphQL */ `
  fragment ExplorerTransferFragment on v0_transfers {
      sender
      source_chain_id
      source_timestamp
      source_transaction_hash
      receiver
      destination_chain_id
      destination_timestamp
      destination_transaction_hash
      assets

      forwards {
        port
        channel
        receiver
        chain { chain_id }
      }
  }
`)
