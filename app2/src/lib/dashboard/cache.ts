import { Effect, Option, pipe } from "effect";
import { UnknownException } from "./errors";

type LocalCacheEntry<A> = {
    value: A;
    expiresAt: number;
    fetchedAt: number;
  };
  

const now = () => Date.now();

const getCacheKey = (namespace: string, key: string) =>
  `cache:${namespace}:${key}`;

const readCache = <A>(
    namespace: string,
    key: string
  ): Effect.Effect<Option.Option<LocalCacheEntry<A>>, UnknownException, never> =>
    pipe(
      Effect.try(() => localStorage.getItem(getCacheKey(namespace, key))),
      Effect.map(Option.fromNullable),
      Effect.flatMap((rawOpt) =>
        pipe(
          rawOpt,
          Option.match({
            onNone: () => Effect.succeed(Option.none()),
            onSome: (raw) =>
              pipe(
                Effect.try(() => JSON.parse(raw) as LocalCacheEntry<A>),
                Effect.map(Option.some),
                Effect.catchAll((cause) =>
                  Effect.fail(new UnknownException({ message: "Failed to parse localStorage entry", cause }))
                )
              ),
          })
        )
      ),
      Effect.catchAll((cause) =>
        Effect.fail(new UnknownException({ message: "Failed to access localStorage", cause }))
      )
    );

const GLOBAL_LAST_SYNC_KEY = "cache:lastSyncTime";

const writeCache = <A>(
    namespace: string,
    key: string,
    value: A,
    ttlMs: number
): Effect.Effect<void, never, never> =>
    Effect.sync(() => {
        const entry = {
        value,
        expiresAt: now() + ttlMs,
        fetchedAt: now(),
    };
        localStorage.setItem(getCacheKey(namespace, key), JSON.stringify(entry));
        localStorage.setItem(GLOBAL_LAST_SYNC_KEY, entry.fetchedAt.toString());
});
    
      

export function withLocalStorageCacheStale<A>(
  namespace: string,
  key: string,
  ttlMs: number,
  staleMs: number,
  effect: Effect.Effect<A, never, never>
): Effect.Effect<A, UnknownException, never> {
  return pipe(
    readCache<A>(namespace, key),
    Effect.flatMap((entryOpt) =>
      pipe(
        entryOpt,
        Option.match({
          onNone: () =>
            pipe(
              effect,
              Effect.tap((result) =>
                writeCache(namespace, key, result, ttlMs)
              )
            ),

          onSome: (entry) => {
            const age = now();
            const isFresh = entry.expiresAt > age;
            const isStale = entry.expiresAt <= age && entry.expiresAt + staleMs > age;

            if (isFresh) {
              return Effect.succeed(entry.value);
            }

            if (isStale) {
              // async revalidate
              Effect.runFork(
                pipe(
                  effect,
                  Effect.tap((result) =>
                    writeCache(namespace, key, result, ttlMs)
                  )
                )
              );
              return Effect.succeed(entry.value); // return stale
            }

            // fully expired
            return pipe(
              effect,
              Effect.tap((result) =>
                writeCache(namespace, key, result, ttlMs)
              )
            );
          },
        })
      )
    )
  );
}

export function getCacheMetadata(
    namespace: string,
    key: string
  ): Option.Option<{ fetchedAt: number; ageMs: number }> {
    const raw = localStorage.getItem(getCacheKey(namespace, key));
    if (!raw) return Option.none();
  
    try {
      const parsed = JSON.parse(raw) as { fetchedAt: number };
      return Option.some({
        fetchedAt: parsed.fetchedAt,
        ageMs: Date.now() - parsed.fetchedAt,
      });
    } catch {
      return Option.none();
    }
  }

  export function getLastGlobalSync(locale: string = "default"): Option.Option<{ label: string; timestamp: number }> {
    const raw = localStorage.getItem("cache:lastSyncTime");
    console.log("raw timestamp:", raw);
    if (!raw) return Option.none();
  
    const timestamp = parseInt(raw, 10);
    console.log("parsed timestamp:", timestamp);
    if (isNaN(timestamp)) return Option.none();
  
    const formatter = new Intl.DateTimeFormat(locale, {
      hour: "2-digit",
      minute: "2-digit",
    });
  
    const label = `Refreshed at: ${formatter.format(new Date(timestamp))}`;
    console.log("formatted label:", label);
  
    return Option.some({ label, timestamp });
  }
  
  
  
