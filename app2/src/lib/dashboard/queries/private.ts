import { extractErrorDetails } from "@unionlabs/sdk/utils"
import { Effect, Option, pipe } from "effect"
import { getSupabaseClient } from "../client"
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
import { retryForever } from "./retry"
import { errorStore } from "../stores/errors.svelte"

export type UserAchievement = Entity<"user_achievements">
export type UserExperience = Entity<"leaderboard">
export type UserMission = Entity<"user_missions">
export type UserReward = Entity<"user_rewards_with_queue">
export type Wallet = Entity<"wallets">

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
          try: () => client.from("user_achievements").select("*").eq("user_id", userId),
          catch: (error) => new SupabaseError({ 
            operation: "loadUserAchievements",
            cause: extractErrorDetails(error as Error) 
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
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () => client.from("leaderboard").select("*").eq("user_id", userId).single(),
          catch: (error) => new SupabaseError({ 
            operation: "loadUserExperience",
            cause: extractErrorDetails(error as Error) 
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
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () => client.from("user_missions").select("*").eq("user_id", userId),
          catch: (error) => new SupabaseError({ 
            operation: "loadUserMissions",
            cause: extractErrorDetails(error as Error) 
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
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client.from("user_rewards_with_queue").select("*").eq("user_id", userId).order(
              "created_at",
              { ascending: false },
            ),
          catch: (error) => new SupabaseError({ 
            operation: "loadUserRewards",
            cause: extractErrorDetails(error as Error) 
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
      getSupabaseClient(),
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client.from("wallets").select("*").eq("user_id", userId).order("created_at", {
              ascending: false,
            }),
          catch: (error) => new SupabaseError({ 
            operation: "loadUserWallets",
            cause: extractErrorDetails(error as Error) 
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
    getSupabaseClient(),
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () => client.from("wallets").delete().eq("user_id", userId).eq("address", address),
        catch: (error) => new SupabaseError({ 
          operation: "removeWallet",
          cause: extractErrorDetails(error as Error) 
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
        Effect.logInfo(
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
    getSupabaseClient(),
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () => client.functions.invoke("tick", { body: { user_id: userId } }),
        catch: (error) => new SupabaseError({ 
          operation: "invokeTick",
          cause: extractErrorDetails(error as Error) 
        }),
      })
    ),
    Effect.retry(retryForever),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ operation: "invokeTick", cause: error }))
      return Effect.succeed(void 0)
    }),
  )

interface SubmitWalletVerificationInput {
  id: string
  address: string
  chainId: string
  message: string
  signature: string
  selectedChains: Array<string | null>
}

export const submitWalletVerification = (
  input: SubmitWalletVerificationInput,
) =>
  pipe(
    getSupabaseClient(),
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
          Effect.fail(new SupabaseError({ operation: "verifyWallet", cause: extractErrorDetails(error as Error) }))
        ),
      )
    }),
  )
