import { dailyTransfers, statistics } from "$lib/stores/statistics.svelte"
import { createQueryGraphql } from "$lib/utils/queries"
import { DailyTransfers, Statistics } from "@unionlabs/sdk/schema"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"

export const statisticsQuery = createQueryGraphql({
  schema: Schema.Struct({ v2_stats_count: Statistics }),
  document: graphql(`
      query StatsQuery @cached(ttl: 1) {
        v2_stats_count {
          name
          value
        }
      }
    `),
  variables: {},
  refetchInterval: "1 second",
  writeData: data => {
    statistics.data = data.pipe(Option.map(d => d.v2_stats_count))
  },
  writeError: error => {
    statistics.error = error
  },
})

export const dailyTransfersQuery = (limit = 30) =>
  createQueryGraphql({
    schema: Schema.Struct({ v2_stats_transfers_daily_count: DailyTransfers }),
    document: graphql(`
      query TransfersPerDay($limit: Int!) @cached(ttl: 60) {
        v2_stats_transfers_daily_count(args: { p_days_back: $limit }) {
          count
          day_date
        }
      }
    `),
    variables: { limit },
    refetchInterval: "60 seconds",
    writeData: data => {
      // Only show testnet 10 transfers
      dailyTransfers.data = data.pipe(
        Option.map(d => {
          const modifiedData = [...d.v2_stats_transfers_daily_count]
          const len = modifiedData.length
          for (let i = Math.max(0, len - 10); i < len; i++) {
            modifiedData[i] = { ...modifiedData[i], count: 0 }
          }
          return modifiedData
        }),
      )
    },
    writeError: error => {
      dailyTransfers.error = error
    },
  })
