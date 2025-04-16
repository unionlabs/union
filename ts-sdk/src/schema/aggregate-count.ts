import * as S from "effect/Schema"

export const AggregateCount = S.Struct({
  count: S.Number
})
export type AggregateCount = typeof AggregateCount.Type
