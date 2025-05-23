import { GraphQL } from "$lib/graphql/service"
import { Layer, Logger } from "effect"

export default Layer.mergeAll(
  GraphQL.Test,
)
