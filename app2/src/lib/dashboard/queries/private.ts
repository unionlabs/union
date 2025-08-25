import { extractErrorDetails } from "@unionlabs/sdk/utils/index"
import { Effect, Option, pipe } from "effect"
import { SupabaseClient } from "../client"
import type { Entity } from "../client"
import { CACHE_VERSION, STALE, TTL } from "../config"
import {
  AchievementError,
  LeaderboardError,
  MissionError,
  RewardError,
  SupabaseError,
  WalletError,
} from "../errors"
import { clearLocalStorageCacheEntry, withLocalStorageCacheStale } from "../services/cache"
import { errorStore } from "../stores/errors.svelte"
import { retryForever } from "./retry"

export type UserAchievement = Entity<"user_achievements">
export type UserExperience = Entity<"leaderboard">
export type UserMission = Entity<"user_missions">
export type UserReward = Entity<"user_rewards_with_queue">
export type Wallet = Entity<"wallets">
export type Device = Entity<"devices">
export type YapsSeason = {
  user_id: string | null
  username: string | null
  mindshare: string | null
  twitter_id: number | null
  pfp: string | null
  team: boolean | null
  rank: number | null
}

type DeviceInsert = {
  ipAddress: string
  userId: string
  deviceIdentifier: string
}

export const getUserAchievements = (userId: string) =>
  withLocalStorageCacheStale(
    "user_achievements",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      SupabaseClient,
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () => client.from("user_achievements").select("*").eq("user_id", userId),
          catch: (error) =>
            new SupabaseError({
              operation: "loadUserAchievements",
              cause: extractErrorDetails(error as Error),
            }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll((error) => {
        errorStore.showError(new AchievementError({ cause: error, operation: "load" }))
        return Effect.succeed(Option.none())
      }),
    ),
  )

export const getUserExperience = (userId: string) =>
  withLocalStorageCacheStale(
    "user_experience",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      SupabaseClient,
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () => client.from("leaderboard").select("*").eq("user_id", userId).single(),
          catch: (error) =>
            new SupabaseError({
              operation: "loadUserExperience",
              cause: extractErrorDetails(error as Error),
            }),
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
          })
        }
        return Option.fromNullable(data)
      }),
      Effect.catchAll((error) => {
        errorStore.showError(new LeaderboardError({ cause: error, operation: "load" }))
        return Effect.succeed(Option.none())
      }),
    ),
  )

export const getUserMissions = (userId: string) =>
  withLocalStorageCacheStale(
    "user_missions",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      SupabaseClient,
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () => client.from("user_missions").select("*").eq("user_id", userId),
          catch: (error) =>
            new SupabaseError({
              operation: "loadUserMissions",
              cause: extractErrorDetails(error as Error),
            }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll((error) => {
        errorStore.showError(new MissionError({ cause: error, operation: "load" }))
        return Effect.succeed(Option.none())
      }),
    ),
  )

export const getUserRewards = (userId: string) =>
  withLocalStorageCacheStale(
    "user_rewards",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      SupabaseClient,
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client.from("user_rewards_with_queue").select("*").eq("user_id", userId).order(
              "created_at",
              { ascending: false },
            ),
          catch: (error) =>
            new SupabaseError({
              operation: "loadUserRewards",
              cause: extractErrorDetails(error as Error),
            }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll((error) => {
        errorStore.showError(new RewardError({ cause: error, operation: "load" }))
        return Effect.succeed(Option.none())
      }),
    ),
  )

export const getWalletsByUserId = (userId: string) =>
  withLocalStorageCacheStale(
    "wallets",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      SupabaseClient,
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client.from("wallets").select("*").eq("user_id", userId).order("created_at", {
              ascending: false,
            }),
          catch: (error) =>
            new SupabaseError({
              operation: "loadUserWallets",
              cause: extractErrorDetails(error as Error),
            }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll((error) => {
        errorStore.showError(new WalletError({ cause: error, operation: "load" }))
        return Effect.succeed(Option.none())
      }),
    ),
  )

export const removeUserWallet = (userId: string, address: string) =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () => client.from("wallets").delete().eq("user_id", userId).eq("address", address),
        catch: (error) =>
          new SupabaseError({
            operation: "removeWallet",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap(response => {
      if (response.error) {
        errorStore.showError(new WalletError({ cause: response.error, operation: "remove" }))
        return Effect.succeed(false)
      }

      const walletCacheKeySuffix = `${CACHE_VERSION}:${userId}`
      return pipe(
        Effect.logTrace(
          `Database wallet removal successful for user ${userId}, address ${address}. Attempting to clear cache for namespace 'wallets', key suffix: ${walletCacheKeySuffix}`,
        ),
        Effect.flatMap(() => clearLocalStorageCacheEntry("wallets", walletCacheKeySuffix)),
        Effect.map(() => true),
        Effect.catchAll(() => Effect.succeed(true)),
      )
    }),
    Effect.catchAll((error) => {
      errorStore.showError(new WalletError({ cause: error, operation: "remove" }))
      return Effect.succeed(false)
    }),
  )

export const invokeTick = (userId: string) =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () => client.functions.invoke("tick", { body: { user_id: userId } }),
        catch: (error) =>
          new SupabaseError({
            operation: "invokeTick",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ operation: "invokeTick", cause: error }))
      return Effect.void
    }),
  )

export const requestRole = (userId: string, rewardId: string) =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("request-role", {
            body: { user_id: userId, reward_id: rewardId },
          }),
        catch: (error) =>
          new SupabaseError({
            operation: "requestRole",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap(response => {
      if (response.error) {
        const errorDetails = extractErrorDetails(response.error)
        return Effect.zipRight(
          Effect.logError("Request role function returned an error in its response.", {
            error: errorDetails,
          }),
          Effect.fail(new RewardError({ operation: "requestRole", cause: errorDetails })),
        )
      }
      return Effect.succeed(response.data)
    }),
    Effect.catchAll((error) => {
      errorStore.showError(new RewardError({ cause: error, operation: "requestRole" }))
      return Effect.succeed(void 0)
    }),
  )

interface SubmitWalletVerificationInput {
  id: string
  address: string
  chainId: string
  message: string
  signature: string
  selectedChains: Array<string | null> | null
}

export const submitWalletVerification = (
  input: SubmitWalletVerificationInput,
) =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("verify-wallet", {
            method: "POST",
            body: input,
          }),
        catch: (error) => {
          return new SupabaseError({
            operation: "verifyWallet",
            cause: extractErrorDetails(error as Error),
          })
        },
      })
    ),
    Effect.flatMap(response => {
      if (response.error) {
        const errorDetails = extractErrorDetails(response.error)
        return Effect.zipRight(
          Effect.logError("Wallet verification function returned an error in its response.", {
            error: errorDetails,
          }),
          Effect.fail(new SupabaseError({ operation: "verifyWallet", cause: errorDetails })),
        )
      }
      return Effect.succeed(response.data)
    }),
    Effect.catchAll((error) => {
      return pipe(
        Effect.logError("Error in submitWalletVerification pipeline.", {
          error: extractErrorDetails(error as Error),
        }),
        Effect.flatMap(() =>
          Effect.fail(
            new SupabaseError({
              operation: "verifyWallet",
              cause: extractErrorDetails(error as Error),
            }),
          )
        ),
      )
    }),
  )

export const getYapsSeason0 = (userId: string) =>
  withLocalStorageCacheStale(
    "yaps_season_0",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      SupabaseClient,
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client
              .from("yaps_season_zero_with_users")
              .select("user_id, username, mindshare, twitter_id, pfp, team, rank")
              .eq("user_id", userId)
              .order("rank", { ascending: true }),
          catch: (error) =>
            new SupabaseError({
              operation: "loadYapsSeason0",
              cause: extractErrorDetails(error as Error),
            }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll((error) => {
        errorStore.showError(new LeaderboardError({ cause: error, operation: "loadYapsSeason0" }))
        return Effect.succeed(Option.none())
      }),
    ),
  )

export const getYapsSeason1 = (userId: string) =>
  withLocalStorageCacheStale(
    "yaps_season_1",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      SupabaseClient,
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client
              .from("yaps_season_one_with_users")
              .select("user_id, username, mindshare, twitter_id, pfp, team, rank")
              .eq("user_id", userId)
              .order("rank", { ascending: true }),
          catch: (error) =>
            new SupabaseError({
              operation: "loadYapsSeason1",
              cause: extractErrorDetails(error as Error),
            }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll((error) => {
        errorStore.showError(new LeaderboardError({ cause: error, operation: "loadYapsSeason1" }))
        return Effect.succeed(Option.none())
      }),
    ),
  )

export const createSnagUserDevice = (input: DeviceInsert) => {
  // Validate all required fields are present
  if (!input.userId || !input.deviceIdentifier || !input.ipAddress) {
    return Effect.succeed(null)
  }

  return pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client
            .from("devices")
            .upsert({
              ipAddress: input.ipAddress,
              userId: input.userId,
              deviceIdentifier: input.deviceIdentifier,
            }),
        catch: (error) =>
          new SupabaseError({
            operation: "upsertDevice",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap(response => {
      if (response.error) {
        const errorDetails = extractErrorDetails(response.error)
        return Effect.zipRight(
          Effect.logError("Device registration function returned an error in its response.", {
            error: errorDetails,
          }),
          Effect.fail(
            new SupabaseError({ operation: "createSnagUserDevice", cause: errorDetails }),
          ),
        )
      }
      return Effect.succeed(response.data)
    }),
    Effect.catchAll((error) => {
      return pipe(
        Effect.logError("Error in createSnagUserDevice pipeline.", {
          error: extractErrorDetails(error as Error),
        }),
        Effect.flatMap(() =>
          Effect.fail(
            new SupabaseError({
              operation: "createSnagUserDevice",
              cause: extractErrorDetails(error as Error),
            }),
          )
        ),
      )
    }),
  )
}
