import { graphql } from "gql.tada"

export const unionToUnionTransfersQuery = graphql(/* GraphQL */ `
  query UnionToUnionTransfers($address: String!, $limit: Int! = 5) {
    v0_transfers(
        limit: $limit,
        where: { _or: [{ sender: { _eq: $address }}, { recipient: { _eq: $address }}]}
    ) {
      sender
      recipient
      amount
      denom
      height
      chain_id
      transaction_hash
    }
  }
`)
