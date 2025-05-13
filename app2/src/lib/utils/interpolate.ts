import { Array as A, Record as R, String as S } from "effect"
import { dual, pipe } from "effect/Function"

type ExtractPlaceholders<T extends string> = T extends
  `${infer _Start}{{${infer Key}}}${infer Rest}` ? Key | ExtractPlaceholders<Rest>
  : never

export const interpolate: {
  <T extends string, Keys extends ExtractPlaceholders<T>>(
    self: T,
  ): (map: { [K in Keys]: string }) => string
  <T extends string, Keys extends ExtractPlaceholders<T>>(
    self: T,
    map: { [K in Keys]: string },
  ): string
} = dual(
  2,
  <T extends string, Keys extends ExtractPlaceholders<T>>(
    self: T,
    map: { [K in Keys]: string },
  ): string =>
    pipe(
      R.map(map, (a, key) => S.replaceAll(`{{${key}}}`, a)),
      R.values,
      A.reduce(self as string, (acc, f) => f(acc)),
    ),
)
