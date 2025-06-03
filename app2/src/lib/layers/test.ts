import { GasPriceMap } from "$lib/gasprice"
import { GraphQL } from "$lib/graphql/service"
import { PriceOracle } from "@unionlabs/sdk/PriceOracle"
import { Layer } from "effect"

export default Layer.mergeAll(
  GraphQL.Test,
  PriceOracle.Test,
  GasPriceMap.Default, // TODO: replace with mock
)
