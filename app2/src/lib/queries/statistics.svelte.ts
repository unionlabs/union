import { Statistics, DailyTransfers } from "$lib/schema/statistics"
import { createQueryGraphql } from "$lib/utils/queries"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { statistics, dailyTransfers } from "$lib/stores/statistics.svelte"

export const statisticsQuery = createQueryGraphql({
  schema: Schema.Struct({ v1_ibc_union_statistics: Statistics }),
  document: graphql(`
      query StatsQuery @cached(ttl: 1) {
        v1_ibc_union_statistics {
          name
          value
        }
      }
    `),
  variables: {},
  refetchInterval: "1 second",
  writeData: data => {
    statistics.data = data.pipe(Option.map(d => d.v1_ibc_union_statistics))
  },
  writeError: error => {
    statistics.error = error
  }
})

export const dailyTransfersQuery = (limit = 30) =>
  createQueryGraphql({
    schema: Schema.Struct({ v1_ibc_union_daily_fungible_asset_orders: DailyTransfers }),
    document: graphql(`
      query TransfersPerDay($limit: Int!) @cached(ttl: 60) {
        v1_ibc_union_daily_fungible_asset_orders(limit: $limit, order_by: {day: desc}) {
          count
          day
        }
      }
    `),
    variables: { limit },
    refetchInterval: "60 seconds",
    writeData: data => {
      dailyTransfers.data = data.pipe(Option.map(d => d.v1_ibc_union_daily_fungible_asset_orders))
    },
    writeError: error => {
      dailyTransfers.error = error
    }
  })
