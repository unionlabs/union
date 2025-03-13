import { Schema } from "effect"

export const StatisticItem = Schema.Struct({
  name: Schema.String,
  value: Schema.NumberFromString
})

export type StatisticItem = Schema.Schema.Type<typeof StatisticItem>

export const Statistics = Schema.Array(StatisticItem)
export type Statistics = Schema.Schema.Type<typeof Statistics>

export const DailyTransfer = Schema.Struct({
  day_date: Schema.DateTimeUtc,
  count: Schema.NumberFromString
})

export type DailyTransfer = Schema.Schema.Type<typeof DailyTransfer>

export const DailyTransfers = Schema.Array(DailyTransfer)
export type DailyTransfers = Schema.Schema.Type<typeof DailyTransfers>
