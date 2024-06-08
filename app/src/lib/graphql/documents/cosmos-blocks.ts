import { graphql } from "gql.tada"

export const cosmosBlocksQuery = graphql(/* GraphQL */ `
  query CosmosBlocksQuery($limit: Int = 20) @cached(ttl: 1) {
    data: v0_blocks(order_by: {time: desc}, limit: $limit) {
      chain_id
      hash
      height
      time
    }
  }
`)

export const cosmosBlocksSubscription = graphql(/* GraphQL */ `
  subscription CosmosBlocksSubscription($limit: Int = 10) {
    data: v0_blocks(order_by: { time: desc }, limit: $limit) {
      chain_id
      hash
      height
      time
    }
  }
`)
