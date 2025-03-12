import { Data, Schema } from "effect"

// Schema for statistics data
export const StatisticItem = Schema.Struct({
  name: Schema.String,
  value: Schema.Int
})

export type StatisticItem = Schema.Schema.Type<typeof StatisticItem>

export const Statistics = Schema.Array(StatisticItem)
export type Statistics = Schema.Schema.Type<typeof Statistics>

// Schema for daily transfers data
export const DailyTransfer = Schema.Struct({
  day: Schema.DateTimeUtc,
  count: Schema.Number
})

export type DailyTransfer = Schema.Schema.Type<typeof DailyTransfer>

export const DailyTransfers = Schema.Array(DailyTransfer)
export type DailyTransfers = Schema.Schema.Type<typeof DailyTransfers>

// Error types
export class FetchStatisticsError extends Data.TaggedError("FetchStatisticsError")<{
  cause: unknown
}> {}

export class FetchDailyTransfersError extends Data.TaggedError("FetchDailyTransfersError")<{
  cause: unknown
}> {}
