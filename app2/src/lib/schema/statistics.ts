import { Schema } from "effect"

export const StatisticItem = Schema.Struct({
  name: Schema.String,
  value: Schema.Int
})

export type StatisticItem = Schema.Schema.Type<typeof StatisticItem>

export const Statistics = Schema.Array(StatisticItem)
export type Statistics = Schema.Schema.Type<typeof Statistics>

export const DailyTransfer = Schema.Struct({
  day: Schema.DateTimeUtc,
  count: Schema.Int
})

export type DailyTransfer = Schema.Schema.Type<typeof DailyTransfer>

export const DailyTransfers = Schema.Array(DailyTransfer)
export type DailyTransfers = Schema.Schema.Type<typeof DailyTransfers>
