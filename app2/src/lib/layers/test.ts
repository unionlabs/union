import { GraphQL } from "$lib/graphql/service"
import { PriceOracle } from "@unionlabs/sdk/PriceOracle"
import { Layer, Logger } from "effect"

export default Layer.mergeAll(
  GraphQL.Test,
  PriceOracle.Test,
)
