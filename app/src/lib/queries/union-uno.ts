import { graphql } from "gql.tada"

export const unionToUnionTransfersQuery = graphql(/* GraphQL */ `
  query UnionToUnionTransfers($address: String!, $limit: Int! = 5) {
    v0_transfers(
        limit: $limit,
        where: { _or: [{ sender: { _eq: $address }}, { receiver: { _eq: $address }}]}
    ) {
      sender
      receiver
      source_port
      source_time
      source_height
      source_channel
      source_sequence
      source_chain_id
      source_block_hash
      source_transaction_hash
      source_transaction_index
      source_timeout_timestamp
      destination_port
      destination_time
      destination_height
      destination_channel
      destination_sequence
      destination_chain_id
      destination_block_hash
      destination_transaction_hash
      destination_transaction_index
      destination_timeout_timestamp
    }
  }
`)
