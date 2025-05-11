import { Effect, pipe, Option } from "effect";
import { getSupabaseClient } from "../client";
import { SupabaseError } from "../errors";
import { retryForever } from "./retry";
import { withLocalStorageCacheStale } from "../cache";
import type { Entity } from "../client";
import { extractErrorDetails } from "@unionlabs/sdk/utils";

export type Achievement = Entity<"achievements">;
export type Level = Entity<"levels">;
export type Category = Entity<"categories">;
export type UserExperience = Entity<"user_levels">;
export type Mission = Entity<"missions">;
export type Reward = Entity<"rewards">;

const TTL = 5 * 60_000;
const STALE = 15 * 60_000;
const CACHE_VERSION = "v1";

export const getChains = () =>
  withLocalStorageCacheStale(
    "public",
    `${CACHE_VERSION}:chains`,
    TTL,
    STALE,
    pipe(
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () => client.from("chains").select("*"),
          catch: (error) =>
            new SupabaseError({ cause: extractErrorDetails(error as Error) }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll(() => Effect.succeed(Option.none()))
    )
  );

export const getAvailableAchievements = () =>
  withLocalStorageCacheStale(
    "public",
    `${CACHE_VERSION}:achievements`,
    TTL,
    STALE,
    pipe(
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client
              .from("achievements")
              .select(`
                *,
                reward_achievements(rewards(*)),
                category:categories!achievements_category_fkey(id, title),
                subcategory:categories!achievements_subcategory_fkey(id, title)
              `),
          catch: (error) =>
            new SupabaseError({ cause: extractErrorDetails(error as Error) }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll(() => Effect.succeed(Option.none()))
    )
  );

export const getAvailableLevels = () =>
  withLocalStorageCacheStale(
    "public",
    `${CACHE_VERSION}:levels`,
    TTL,
    STALE,
    pipe(
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client
              .from("levels")
              .select("*")
              .order("experience_required", { ascending: true }),
          catch: (error) =>
            new SupabaseError({ cause: extractErrorDetails(error as Error) }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll(() => Effect.succeed(Option.none()))
    )
  );

export const getCategories = () =>
  withLocalStorageCacheStale(
    "public",
    `${CACHE_VERSION}:categories`,
    TTL,
    STALE,
    pipe(
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () => client.from("categories").select("*"),
          catch: (error) =>
            new SupabaseError({ cause: extractErrorDetails(error as Error) }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll(() => Effect.succeed(Option.none()))
    )
  );

export const getLeaderboard = () =>
  withLocalStorageCacheStale(
    "public",
    `${CACHE_VERSION}:leaderboard`,
    TTL,
    STALE,
    pipe(
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client
              .from("user_levels")
              .select("*")
              .order("total_xp", { ascending: false })
              .limit(50),
          catch: (error) =>
            new SupabaseError({ cause: extractErrorDetails(error as Error) }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll(() => Effect.succeed(Option.none()))
    )
  );

export const getAvailableMissions = () =>
  withLocalStorageCacheStale(
    "public",
    `${CACHE_VERSION}:missions`,
    TTL,
    STALE,
    pipe(
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client
              .from("missions")
              .select("*")
              .order("priority", { ascending: false }),
          catch: (error) =>
            new SupabaseError({ cause: extractErrorDetails(error as Error) }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll(() => Effect.succeed(Option.none()))
    )
  );

export const getAvailableRewards = () =>
  withLocalStorageCacheStale(
    "public",
    `${CACHE_VERSION}:rewards`,
    TTL,
    STALE,
    pipe(
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () => client.from("rewards").select("*"),
          catch: (error) =>
            new SupabaseError({ cause: extractErrorDetails(error as Error) }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll(() => Effect.succeed(Option.none()))
    )
  );
