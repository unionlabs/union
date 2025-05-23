import { Effect, HashSet, ParseResult, pipe, Schema as S } from "effect"
import { apply } from "effect/Function"
import { isObject } from "effect/Predicate"

/**
 * Handles circular JSON by replacing the value with `null`
 */
export const safeStringifyJSON = (x: unknown): Effect.Effect<string, ParseResult.ParseError> =>
  Effect.gen(function*() {
    /**
     * This is valid for 1-indexed `JSON.strigify` parameter, but this cannot be reliably extracted from the types given function overloading.
     */
    type Replacer = (this: any, key: string, value: any) => any

    let visited = HashSet.empty<object>()

    const replacer: Replacer = (_, value) => {
      if (isObject(value)) {
        if (HashSet.has(visited, value)) {
          return "<circular>"
        }
        visited = HashSet.add(visited, value)
      }
      return value
    }

    return yield* pipe(
      // TODO: make spacing configurable
      S.parseJson({ replacer: replacer as any, space: 2 }),
      S.encodeUnknown,
      apply(x),
    )
  })
