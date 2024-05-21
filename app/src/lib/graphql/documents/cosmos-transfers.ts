import { graphql } from "gql.tada"

export const cosmosTransfersQuery = graphql(/* GraphQL */ `
  query CosmosUnionTransfersQuery($address: String!, $limit: Int!) {
    data: v0_cosmos_transfer(
      limit:  $limit,
      order_by: { height: desc },
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

export const cosmosTransfersSubscription = graphql(/* GraphQL */ `
  subscription CosmosUnionTransfersSubscription($address: String!, $limit: Int!) {
    data: v0_cosmos_transfer(
      limit:  $limit,
      order_by: { height: desc },
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
