import { graphql } from "gql.tada"

export const allTransfersQueryDocument = graphql(/* GraphQL */ `
  query AllTransfersQuery @cached(ttl: 1) {
    v0_transfers(limit: 50, order_by: {source_timestamp: desc}) {
      sender
      normalized_sender
      source_chain_id
      source_connection_id
      source_channel_id
      receiver
      normalized_receiver
      destination_chain_id
      destination_connection_id
      destination_channel_id
      assets
      source_timestamp
      destination_timestamp
    }
  }
`)
