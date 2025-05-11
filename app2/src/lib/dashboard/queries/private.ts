import { Effect, pipe, Option } from "effect";
import { getSupabaseClient } from "../client";
import { SupabaseError } from "../errors";
import { retryForever } from "./retry";
import { withLocalStorageCacheStale } from "../cache";
import type { Entity } from "../client";
import { extractErrorDetails } from "@unionlabs/sdk/utils";

export type UserAchievement = Entity<"user_achievements">;
export type UserExperience = Entity<"leaderboard">;
export type UserMission = Entity<"user_missions">;
export type UserReward = Entity<"user_rewards_with_queue">;
export type Wallet = Entity<"wallets">;

const TTL = 5 * 60_000;         // 5 minutes
const STALE = 24 * 60 * 60_000; // 24 hours
const CACHE_VERSION = "v1";

export const getUserAchievements = (userId: string) =>
  withLocalStorageCacheStale(
    "user_achievements",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client.from("user_achievements").select("*").eq("user_id", userId),
          catch: (error) =>
            new SupabaseError({ cause: extractErrorDetails(error as Error) }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll(() => Effect.succeed(Option.none()))
    )
  );

export const getUserExperience = (userId: string) =>
  withLocalStorageCacheStale(
    "user_experience",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client
              .from("leaderboard")
              .select("*")
              .eq("user_id", userId)
              .single(),
          catch: (error) =>
            new SupabaseError({ cause: extractErrorDetails(error as Error) }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data, error }) => {
        if (!data && error?.code === "PGRST116") {
          return Option.some({
            user_id: userId,
            current_xp: 0,
            total_xp: 0,
            level: 1,
            xp_required: 30,
            rank: 999999,
            title: "Conscript",
            display_name: null,
            pfp: null,
          });
        }
        return Option.fromNullable(data);
      }),
      Effect.catchAll(() => Effect.succeed(Option.none()))
    )
  );

export const getUserMissions = (userId: string) =>
  withLocalStorageCacheStale(
    "user_missions",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client.from("user_missions").select("*").eq("user_id", userId),
          catch: (error) =>
            new SupabaseError({ cause: extractErrorDetails(error as Error) }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll(() => Effect.succeed(Option.none()))
    )
  );

export const getUserRewards = (userId: string) =>
  withLocalStorageCacheStale(
    "user_rewards",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client
              .from("user_rewards_with_queue")
              .select("*")
              .eq("user_id", userId)
              .order("created_at", { ascending: false }),
          catch: (error) =>
            new SupabaseError({ cause: extractErrorDetails(error as Error) }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll(() => Effect.succeed(Option.none()))
    )
  );

export const getWalletsByUserId = (userId: string) =>
  withLocalStorageCacheStale(
    "wallets",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client
              .from("wallets")
              .select("*")
              .eq("user_id", userId)
              .order("created_at", { ascending: false }),
          catch: (error) =>
            new SupabaseError({ cause: extractErrorDetails(error as Error) }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll(() => Effect.succeed(Option.none()))
    )
  );

export const removeUserWallet = (userId: string, address: string) =>
  pipe(
    getSupabaseClient(),
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client
            .from("wallets")
            .delete()
            .eq("user_id", userId)
            .eq("address", address),
        catch: (error) =>
          new SupabaseError({ cause: extractErrorDetails(error as Error) }),
      })
    ),
    Effect.retry(retryForever),
    Effect.map(({ error }) => {
      if (error) {
        console.error("Error removing user wallet:", error);
        return false;
      }
      return true;
    }),
    Effect.catchAll(() => Effect.succeed(false))
  );

export const insertWalletData = (data: { address: string; chain_id: string; user_id: string }) =>
  pipe(
    getSupabaseClient(),
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client
            .from("wallets")
            .select()
            .eq("address", data.address)
            .single(),
        catch: (error) =>
          new SupabaseError({ cause: extractErrorDetails(error as Error) }),
      })
    ),
    Effect.flatMap((response) => {
      if (response.error && response.error.code !== "PGRST116") {
        return Effect.fail(new SupabaseError({ cause: extractErrorDetails(response.error as Error) }));
      }

      if (response.data) {
        console.log("Wallet already exists");
        return Effect.succeed(response.data);
      }

      return pipe(
        getSupabaseClient(),
        Effect.flatMap((client) =>
          Effect.tryPromise({
            try: () =>
              client
                .from("wallets")
                .insert({
                  address: data.address,
                  chain_id: data.chain_id,
                  user_id: data.user_id,
                })
                .select()
                .single(),
            catch: (error) =>
              new SupabaseError({ cause: extractErrorDetails(error as Error) }),
          })
        ),
        Effect.map((response) => response.data)
      );
    }),
    Effect.retry(retryForever),
    Effect.catchAll(() => Effect.succeed(null))
  );

export const invokeTick = (userId: string) =>
  pipe(
    getSupabaseClient(),
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("tick", {
            body: { user_id: userId },
          }),
        catch: (error) =>
          new SupabaseError({ cause: extractErrorDetails(error as Error) }),
      })
    ),
    Effect.retry(retryForever),
    Effect.catchAll(() => Effect.succeed(void 0))
  );
