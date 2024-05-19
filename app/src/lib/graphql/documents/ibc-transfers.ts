import { graphql } from "gql.tada"

export const ibcTransfersQuery = graphql(/* GraphQL */ `
  query IBCTransfersQuery(
    $limit: Int! = 100
    $unionAddress: String
    $nonUnionAddress: String
  ) {
    v0_cosmos_wasm_ibc_transfer(
      limit: $limit
      order_by: { height: desc }
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
      height
      time
      json
      memo
      chain_id
      transaction_hash
    }
  }
`)

export const ibcTransfersSubscription = graphql(/* GraphQL */ `
  subscription IBCTransfersSubscription(
    $limit: Int! = 100
    $unionAddress: String
    $nonUnionAddress: String
  ) {
    v0_cosmos_wasm_ibc_transfer(
      limit: $limit
      order_by: { height: desc }
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
        height
        time
        json
        memo
        chain_id
        transaction_hash
    }
  }
`)
