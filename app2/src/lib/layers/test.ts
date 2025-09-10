import { SupabaseClient } from "$lib/dashboard/client"
import { GasPriceMap } from "$lib/gasprice"
import { GraphQL } from "$lib/graphql/service"
import { PriceOracle } from "@unionlabs/sdk"
import { Layer } from "effect"

export default Layer.mergeAll(
  GraphQL.Test,
  PriceOracle.layerTest,
  SupabaseClient.Default(), // TODO: replace with mock
  GasPriceMap.Default, // TODO: replace with mock
)
