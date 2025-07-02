import { type AppContext, runFork } from "$lib/runtime"
import { extractErrorDetails } from "@unionlabs/sdk/utils"
import { Data, Effect, Option, pipe } from "effect"

export class CacheError extends Data.TaggedError("CacheError")<{
  cause: unknown
  operation: "read" | "write" | "clear" | "parse"
}> {}

type LocalCacheEntry<A> = {
  value: A
  expiresAt: number
  fetchedAt: number
}

const now = () => Date.now()

const getCacheKey = (namespace: string, key: string) => `cache:${namespace}:${key}`

const readCache = <A>(
  namespace: string,
  key: string,
): Effect.Effect<Option.Option<LocalCacheEntry<A>>, CacheError, never> =>
  pipe(
    Effect.tryPromise(() => Promise.resolve(localStorage.getItem(getCacheKey(namespace, key)))),
    Effect.map(Option.fromNullable),
    Effect.flatMap((rawOpt) =>
      pipe(
        rawOpt,
        Option.match({
          onNone: () => Effect.succeed(Option.none()),
          onSome: (raw) =>
            pipe(
              Effect.tryPromise(() => Promise.resolve(JSON.parse(raw) as LocalCacheEntry<A>)),
              Effect.map(Option.some),
              Effect.catchAll((error) =>
                Effect.fail(
                  new CacheError({
                    cause: extractErrorDetails(error),
                    operation: "parse",
                  }),
                )
              ),
            ),
        }),
      )
    ),
    Effect.catchAll((error) =>
      Effect.fail(
        new CacheError({
          cause: extractErrorDetails(error),
          operation: "read",
        }),
      )
    ),
  )

const GLOBAL_LAST_SYNC_KEY = "cache:lastSyncTime"

const writeCache = <A>(
  namespace: string,
  key: string,
  value: A,
  ttlMs: number,
): Effect.Effect<void, CacheError, never> =>
  pipe(
    Effect.tryPromise(() => {
      const entry = {
        value,
        expiresAt: now() + ttlMs,
        fetchedAt: now(),
      }
      localStorage.setItem(getCacheKey(namespace, key), JSON.stringify(entry))
      localStorage.setItem(GLOBAL_LAST_SYNC_KEY, entry.fetchedAt.toString())
      return Promise.resolve()
    }),
    Effect.catchAll((error) =>
      Effect.fail(
        new CacheError({
          cause: extractErrorDetails(error),
          operation: "write",
        }),
      )
    ),
  )

export function withLocalStorageCacheStale<A, R extends AppContext>(
  namespace: string,
  key: string,
  ttlMs: number,
  staleMs: number,
  effect: Effect.Effect<A, never, R>,
): Effect.Effect<A, CacheError, R> {
  return pipe(
    readCache<A>(namespace, key),
    Effect.flatMap((entryOpt) =>
      pipe(
        entryOpt,
        Option.match({
          onNone: () =>
            pipe(
              effect,
              Effect.tap((result: A) => {
                if (Option.isOption(result) && Option.isNone(result)) {
                  return Effect.void
                }
                return writeCache(namespace, key, result, ttlMs)
              }),
            ),

          onSome: (entry) => {
            const age = now()
            const isFresh = entry.expiresAt > age
            const isStale = entry.expiresAt <= age && entry.expiresAt + staleMs > age

            if (isFresh) {
              return Effect.succeed(entry.value)
            }

            if (isStale) {
              runFork(
                pipe(
                  effect,
                  Effect.tap((result: A) => {
                    if (Option.isOption(result) && Option.isNone(result)) {
                      return Effect.void
                    }
                    return writeCache(namespace, key, result, ttlMs)
                  }),
                ),
              )
              return Effect.succeed(entry.value)
            }

            return pipe(
              effect,
              Effect.tap((result: A) => {
                if (Option.isOption(result) && Option.isNone(result)) {
                  return Effect.void
                }
                return writeCache(namespace, key, result, ttlMs)
              }),
            )
          },
        }),
      )
    ),
  )
}

export function getCacheMetadata(
  namespace: string,
  key: string,
): Effect.Effect<Option.Option<{ fetchedAt: number; ageMs: number }>, CacheError, never> {
  return pipe(
    Effect.tryPromise(() => Promise.resolve(localStorage.getItem(getCacheKey(namespace, key)))),
    Effect.map(Option.fromNullable),
    Effect.flatMap((rawOpt) =>
      pipe(
        rawOpt,
        Option.match({
          onNone: () => Effect.succeed(Option.none()),
          onSome: (raw) =>
            pipe(
              Effect.tryPromise(() => Promise.resolve(JSON.parse(raw) as { fetchedAt: number })),
              Effect.map((parsed) =>
                Option.some({
                  fetchedAt: parsed.fetchedAt,
                  ageMs: Date.now() - parsed.fetchedAt,
                })
              ),
              Effect.catchAll((error) =>
                Effect.fail(
                  new CacheError({
                    cause: extractErrorDetails(error),
                    operation: "parse",
                  }),
                )
              ),
            ),
        }),
      )
    ),
    Effect.catchAll((error) =>
      Effect.fail(
        new CacheError({
          cause: extractErrorDetails(error),
          operation: "read",
        }),
      )
    ),
  )
}

export function getLastGlobalSync(
  locale: string = "default",
): Effect.Effect<Option.Option<{ label: string; timestamp: number }>, CacheError, never> {
  return pipe(
    Effect.tryPromise(() => Promise.resolve(localStorage.getItem("cache:lastSyncTime"))),
    Effect.map(Option.fromNullable),
    Effect.flatMap((rawOpt) =>
      pipe(
        rawOpt,
        Option.match({
          onNone: () => Effect.succeed(Option.none()),
          onSome: (raw) =>
            pipe(
              Effect.tryPromise(() => Promise.resolve(parseInt(raw, 10))),
              Effect.flatMap((timestamp) => {
                if (isNaN(timestamp)) {
                  return Effect.succeed(Option.none())
                }

                const formatter = new Intl.DateTimeFormat(locale, {
                  hour: "2-digit",
                  minute: "2-digit",
                })

                const label = `Refreshed at: ${formatter.format(new Date(timestamp))}`
                return Effect.succeed(Option.some({ label, timestamp }))
              }),
              Effect.catchAll((error) =>
                Effect.fail(
                  new CacheError({
                    cause: extractErrorDetails(error),
                    operation: "parse",
                  }),
                )
              ),
            ),
        }),
      )
    ),
    Effect.catchAll((error) =>
      Effect.fail(
        new CacheError({
          cause: extractErrorDetails(error),
          operation: "read",
        }),
      )
    ),
  )
}

export const clearLocalStorageCacheEntry = (
  namespace: string,
  key: string,
): Effect.Effect<void, CacheError> => {
  const fullCacheKey = getCacheKey(namespace, key)
  return pipe(
    Effect.logTrace(`Attempting to clear localStorage cache entry for key: ${fullCacheKey}`),
    Effect.flatMap(() =>
      Effect.tryPromise(() => {
        localStorage.removeItem(fullCacheKey)
        return Promise.resolve()
      })
    ),
    Effect.catchAll((error) =>
      Effect.fail(
        new CacheError({
          cause: extractErrorDetails(error),
          operation: "clear",
        }),
      )
    ),
    Effect.tapBoth({
      onFailure: (error) =>
        Effect.logError(
          `Failed to clear cache for key: ${fullCacheKey}`,
          error,
        ),
      onSuccess: () =>
        Effect.logTrace(
          `Successfully cleared localStorage cache entry for key: ${fullCacheKey}`,
        ),
    }),
    Effect.asVoid,
  )
}
