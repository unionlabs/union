import * as S from "effect/Schema"

export const StatisticItem = S.Struct({
  name: S.String,
  value: S.NumberFromString
})
export type StatisticItem = typeof StatisticItem.Type

export const Statistics = S.Array(StatisticItem)
export type Statistics = typeof Statistics.Type

export const DailyTransfer = S.Struct({
  day_date: S.DateTimeUtc,
  count: S.NumberFromString
})
export type DailyTransfer = typeof DailyTransfer.Type

export const DailyTransfers = S.Array(DailyTransfer)
export type DailyTransfers = typeof DailyTransfers.Type
