import { graphql } from "gql.tada"

export const voyagerQueueQuery = graphql(/* GraphQL */ `
  query VoyagerQueue($limit: Int = 10) {
    queue(order_by: {id: desc}, limit: $limit, where: {status: {_eq: "ready"}}) {
      id
      item
      message
      parent
    }
  }
`)
