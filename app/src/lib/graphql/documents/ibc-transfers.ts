import { graphql } from "gql.tada"

export const ibcTransfersQuery = graphql(/* GraphQL */ `
  query IBCTransfersQuery(
    $limit: Int! = 100
    $unionAddress: String
    $nonUnionAddress: String
  ) {
    data: v0_transfers(
      limit: $limit
      order_by: { source_time: desc }
      where: {
        _or: [
          {
            _or: [
              { sender: { _eq: $unionAddress } }
              { receiver: { _eq: $unionAddress } }
            ]
          }
          {
            _or: [
              { sender: { _eq: $nonUnionAddress } }
              { receiver: { _eq: $nonUnionAddress } }
            ]
          }
        ]
      }
    ) {
      source_time
      source_height
      source_chain_id
      source_transaction_hash
      source_channel
      source_sequence
      source_timeout_timestamp
      source_transaction_index

      destination_time
      destination_height
      destination_chain_id
      destination_transaction_hash
      destination_channel
      destination_sequence
      destination_timeout_timestamp
      destination_transaction_index
    }
  }
`)
