import { extractErrorDetails } from "@unionlabs/sdk/utils/index"
import { Effect, Option, pipe } from "effect"
import { type Entity, SupabaseClient } from "../client"
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

import type {
  AcceptTermsResponse,
  ClaimReferralCodeResponse,
  CreateAirdropEntryResponse,
  DeviceInsert,
  GenerateReferralCodeResponse,
  RemoveReferralCodeResponse,
  ScanAllocationResponse,
  SubmitWalletVerificationInput,
  UpdatePreStakeResponse,
  UpdateTwitterResponse,
  VerifyHumanResponse,
  VerifyStargazeWalletResponse,
  VerifyWalletSignatureResponse,
} from "./types"

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

export const getUserAllocation = (userId: string) =>
  withLocalStorageCacheStale(
    "user_allocation",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      SupabaseClient,
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client
              .from("user_allocations")
              .select("*")
              .eq("user_id", userId)
              .single(),
          catch: (error) =>
            new SupabaseError({
              operation: "loadUserAllocation",
              cause: extractErrorDetails(error as Error),
            }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll((error) => {
        // Don't show error for missing user allocation - this is expected for new users
        console.info("User allocation not found (expected for new users)", { userId, error })
        return Effect.succeed(Option.none())
      }),
    ),
  )

/** Update pre-stake amount via edge function */
export const updatePreStake = (
  additionalAmount: number,
): Effect.Effect<Option.Option<UpdatePreStakeResponse>, never, SupabaseClient> =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("airdrop-update-prestake", {
            body: { additional_amount: additionalAmount },
          }),
        catch: (error) =>
          new SupabaseError({
            operation: "updatePreStake",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap(({ data, error }) => {
      if (error) {
        return Effect.fail(
          new SupabaseError({
            operation: "updatePreStake",
            cause: error,
          }),
        )
      }

      // Check if the function returned an error in the response data
      if (data && !data.success && data.error) {
        return Effect.fail(
          new SupabaseError({
            operation: "updatePreStake",
            cause: { message: data.error, name: "EdgeFunctionError" },
          }),
        )
      }

      return Effect.succeed(Option.fromNullable(data as UpdatePreStakeResponse))
    }),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ cause: error, operation: "updatePreStake" }))
      return Effect.succeed(Option.none())
    }),
  )

export const getReferralCodesByUserId = (userId: string) =>
  withLocalStorageCacheStale(
    "referral_codes",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      SupabaseClient,
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client
              .from("referral_codes_with_users")
              .select("*")
              .eq("owner_user_id", userId)
              .order("created_at", { ascending: false }),
          catch: (error) =>
            new SupabaseError({
              operation: "loadUserReferralCodes",
              cause: extractErrorDetails(error as Error),
            }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll((error) => {
        errorStore.showError(new SupabaseError({ cause: error, operation: "loadReferralCodes" }))
        return Effect.succeed(Option.none())
      }),
    ),
  )

export const getClaimedReferralCodesByUserId = (userId: string) =>
  withLocalStorageCacheStale(
    "claimed_referral_codes",
    `${CACHE_VERSION}:${userId}`,
    TTL,
    STALE,
    pipe(
      SupabaseClient,
      Effect.flatMap((client) =>
        Effect.tryPromise({
          try: () =>
            client
              .from("referral_codes_with_users")
              .select("*")
              .eq("claimer_user_id", userId)
              .eq("is_claimed", true)
              .order("claimed_at", { ascending: false }),
          catch: (error) =>
            new SupabaseError({
              operation: "loadClaimedReferralCodes",
              cause: extractErrorDetails(error as Error),
            }),
        })
      ),
      Effect.retry(retryForever),
      Effect.map(({ data }) => Option.fromNullable(data)),
      Effect.catchAll((error) => {
        errorStore.showError(
          new SupabaseError({ cause: error, operation: "loadClaimedReferralCodes" }),
        )
        return Effect.succeed(Option.none())
      }),
    ),
  )

export const getReferralCodeByCode = (code: string) =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client
            .from("referral_codes_with_users")
            .select("*")
            .eq("code", code.toUpperCase())
            .single(),
        catch: (error) =>
          new SupabaseError({
            operation: "loadReferralCodeByCode",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.map(({ data }) => Option.fromNullable(data)),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ cause: error, operation: "loadReferralCodeByCode" }))
      return Effect.succeed(Option.none())
    }),
  )

export const generateReferralCode = (
  userId: string,
  percentage: number,
): Effect.Effect<Option.Option<GenerateReferralCodeResponse>, never, SupabaseClient> =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("airdrop-generate-code", {
            body: { user_id: userId, percentage },
          }),
        catch: (error) =>
          new SupabaseError({
            operation: "generateReferralCode",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap(response => {
      if (response.error) {
        const errorDetails = extractErrorDetails(response.error)
        return Effect.zipRight(
          Effect.logError("Generate referral code function returned an error in its response.", {
            error: errorDetails,
          }),
          Effect.fail(
            new SupabaseError({ operation: "generateReferralCode", cause: errorDetails }),
          ),
        )
      }

      // Check if the function returned an error in the response data
      if (response.data && !response.data.success && response.data.error) {
        return Effect.fail(
          new SupabaseError({
            operation: "generateReferralCode",
            cause: { message: response.data.error, name: "EdgeFunctionError" },
          }),
        )
      }

      return Effect.succeed(Option.fromNullable(response.data as GenerateReferralCodeResponse))
    }),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ cause: error, operation: "generateReferralCode" }))
      return Effect.succeed(Option.none())
    }),
  )

interface ReferralCodeDetails {
  code: string
  percentage: number
  owner_name: string
  owner_avatar: string | null
  is_claimed: boolean
  claimed_by_user_id: string | null
  claimed_at: string | null
  created_at: string
}

export const verifyReferralCode = (
  code: string,
): Effect.Effect<Option.Option<ReferralCodeDetails>, never, SupabaseClient> =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("airdrop-verify-code", {
            body: { code },
          }),
        catch: (error) =>
          new SupabaseError({
            operation: "verifyReferralCode",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap((response) => {
      if (response.error) {
        const errorDetails = extractErrorDetails(response.error)
        return Effect.zipRight(
          Effect.logError("Verify referral code function returned an error in its response.", {
            error: errorDetails,
          }),
          Effect.fail(new SupabaseError({ operation: "verifyReferralCode", cause: errorDetails })),
        )
      }

      // Check if the function returned an error in the response data
      if (response.data && !response.data.success && response.data.error) {
        return Effect.fail(
          new SupabaseError({
            operation: "verifyReferralCode",
            cause: { message: response.data.error, name: "EdgeFunctionError" },
          }),
        )
      }

      return Effect.succeed(Option.fromNullable(response.data?.data))
    }),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ cause: error, operation: "verifyReferralCode" }))
      return Effect.succeed(Option.none())
    }),
  )

export const claimReferralCode = (
  userId: string,
  code: string,
): Effect.Effect<Option.Option<ClaimReferralCodeResponse>, never, SupabaseClient> =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("airdrop-claim-code", {
            body: { user_id: userId, code },
          }),
        catch: (error) =>
          new SupabaseError({
            operation: "claimReferralCode",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap(response => {
      if (response.error) {
        const errorDetails = extractErrorDetails(response.error)
        return Effect.zipRight(
          Effect.logError("Claim referral code function returned an error in its response.", {
            error: errorDetails,
          }),
          Effect.fail(new SupabaseError({ operation: "claimReferralCode", cause: errorDetails })),
        )
      }

      // Check if the function returned an error in the response data
      if (response.data && !response.data.success && response.data.error) {
        return Effect.fail(
          new SupabaseError({
            operation: "claimReferralCode",
            cause: { message: response.data.error, name: "EdgeFunctionError" },
          }),
        )
      }

      return Effect.succeed(Option.fromNullable(response.data as ClaimReferralCodeResponse))
    }),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ cause: error, operation: "claimReferralCode" }))
      return Effect.succeed(Option.none())
    }),
  )

export const removeReferralCode = (
  codeId: string,
): Effect.Effect<Option.Option<RemoveReferralCodeResponse>, never, SupabaseClient> =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("airdrop-remove-code", {
            body: { code_id: codeId },
          }),
        catch: (error) =>
          new SupabaseError({
            operation: "removeReferralCode",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap(response => {
      if (response.error) {
        const errorDetails = extractErrorDetails(response.error)
        return Effect.zipRight(
          Effect.logError("Remove referral code function returned an error in its response.", {
            error: errorDetails,
          }),
          Effect.fail(new SupabaseError({ operation: "removeReferralCode", cause: errorDetails })),
        )
      }

      // Check if the function returned an error in the response data
      if (response.data && !response.data.success && response.data.error) {
        return Effect.fail(
          new SupabaseError({
            operation: "removeReferralCode",
            cause: { message: response.data.error, name: "EdgeFunctionError" },
          }),
        )
      }

      return Effect.succeed(Option.fromNullable(response.data as RemoveReferralCodeResponse))
    }),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ cause: error, operation: "removeReferralCode" }))
      return Effect.succeed(Option.none())
    }),
  )

/** Verify Stargaze wallet signature and save wallet address */
export const verifyStargazeWallet = (
  address: string,
  message: string,
  signature: string,
  signingAddress: string,
): Effect.Effect<Option.Option<VerifyStargazeWalletResponse>, never, SupabaseClient> =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("airdrop-verify-stargaze", {
            body: { address, message, signature, signingAddress },
          }),
        catch: (error) =>
          new SupabaseError({
            operation: "verifyStargazeWallet",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap(response => {
      if (response.error) {
        const errorDetails = extractErrorDetails(response.error)
        return Effect.zipRight(
          Effect.logError("Verify stars wallet function returned an error in its response.", {
            error: errorDetails,
          }),
          Effect.fail(
            new SupabaseError({ operation: "verifyStargazeWallet", cause: errorDetails }),
          ),
        )
      }

      // Check if the function returned an error in the response data
      if (response.data && !response.data.success && response.data.error) {
        return Effect.fail(
          new SupabaseError({
            operation: "verifyStargazeWallet",
            cause: { message: response.data.error, name: "EdgeFunctionError" },
          }),
        )
      }

      return Effect.succeed(Option.fromNullable(response.data as VerifyStargazeWalletResponse))
    }),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ cause: error, operation: "verifyStargazeWallet" }))
      return Effect.succeed(Option.none())
    }),
  )

/** Verify airdrop wallet signature and save wallet address */
export const verifyAirdropWallet = (
  walletAddress: string,
  message: string,
  signature: string,
  chainId: string,
): Effect.Effect<Option.Option<VerifyWalletSignatureResponse>, never, SupabaseClient> =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("airdrop-verify-wallet", {
            body: {
              address: walletAddress,
              chainId: chainId,
              message,
              signature,
            },
          }),
        catch: (error) =>
          new SupabaseError({
            operation: "verifyAirdropWallet",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap(response => {
      if (response.error) {
        const errorDetails = extractErrorDetails(response.error)
        return Effect.zipRight(
          Effect.logError("Verify airdrop wallet function returned an error in its response.", {
            error: errorDetails,
          }),
          Effect.fail(new SupabaseError({ operation: "verifyAirdropWallet", cause: errorDetails })),
        )
      }

      // Check if the function returned an error in the response data
      if (response.data && !response.data.success && response.data.error) {
        return Effect.fail(
          new SupabaseError({
            operation: "verifyAirdropWallet",
            cause: { message: response.data.error, name: "EdgeFunctionError" },
          }),
        )
      }

      return Effect.succeed(Option.fromNullable(response.data as VerifyWalletSignatureResponse))
    }),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ cause: error, operation: "verifyAirdropWallet" }))
      return Effect.succeed(Option.none())
    }),
  )

/** Create airdrop entry */
export const createAirdropEntry = (): Effect.Effect<
  Option.Option<CreateAirdropEntryResponse>,
  never,
  SupabaseClient
> =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("airdrop-create-entry", {
            body: {},
          }),
        catch: (error) =>
          new SupabaseError({
            operation: "createAirdropEntry",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap(response => {
      if (response.error) {
        const errorDetails = extractErrorDetails(response.error)
        return Effect.zipRight(
          Effect.logError("Create airdrop entry function returned an error in its response.", {
            error: errorDetails,
          }),
          Effect.fail(new SupabaseError({ operation: "createAirdropEntry", cause: errorDetails })),
        )
      }

      // Check if the function returned an error in the response data
      if (response.data && !response.data.success && response.data.error) {
        return Effect.fail(
          new SupabaseError({
            operation: "createAirdropEntry",
            cause: { message: response.data.error, name: "EdgeFunctionError" },
          }),
        )
      }

      return Effect.succeed(Option.fromNullable(response.data as CreateAirdropEntryResponse))
    }),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ cause: error, operation: "createAirdropEntry" }))
      return Effect.succeed(Option.none())
    }),
  )

/** Update user allocation with Twitter data */
export const updateTwitterData = (
  twitter_id: string,
  twitter_username?: string,
): Effect.Effect<Option.Option<UpdateTwitterResponse>, never, SupabaseClient> =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("airdrop-update-twitter", {
            body: { twitter_id, twitter_username },
          }),
        catch: (error) =>
          new SupabaseError({
            operation: "updateTwitterData",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap(response => {
      if (response.error) {
        const errorDetails = extractErrorDetails(response.error)
        return Effect.zipRight(
          Effect.logError("Update Twitter data function returned an error in its response.", {
            error: errorDetails,
          }),
          Effect.fail(new SupabaseError({ operation: "updateTwitterData", cause: errorDetails })),
        )
      }

      // Check if the function returned an error in the response data
      if (response.data && !response.data.success && response.data.error) {
        return Effect.fail(
          new SupabaseError({
            operation: "updateTwitterData",
            cause: { message: response.data.error, name: "EdgeFunctionError" },
          }),
        )
      }

      return Effect.succeed(Option.fromNullable(response.data as UpdateTwitterResponse))
    }),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ cause: error, operation: "updateTwitterData" }))
      return Effect.succeed(Option.none())
    }),
  )

/** Scan for user allocation in master sheet */
export const scanAllocation = (
  twitter_id?: string,
  stargaze_address?: string,
): Effect.Effect<Option.Option<ScanAllocationResponse>, never, SupabaseClient> =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("airdrop-scan-allocation", {
            body: { twitter_id, stargaze_address },
          }),
        catch: (error) =>
          new SupabaseError({
            operation: "scanAllocation",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap(response => {
      if (response.error) {
        const errorDetails = extractErrorDetails(response.error)
        return Effect.zipRight(
          Effect.logError("Scan allocation function returned an error in its response.", {
            error: errorDetails,
          }),
          Effect.fail(new SupabaseError({ operation: "scanAllocation", cause: errorDetails })),
        )
      }
      return Effect.succeed(Option.fromNullable(response.data as ScanAllocationResponse))
    }),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ cause: error, operation: "scanAllocation" }))
      return Effect.succeed(Option.none())
    }),
  )

/** Accept terms of service */
export const acceptTerms = (
  accepted: boolean,
): Effect.Effect<Option.Option<AcceptTermsResponse>, never, SupabaseClient> =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("airdrop-accept-terms", {
            body: { accepted },
          }),
        catch: (error) =>
          new SupabaseError({
            operation: "acceptTerms",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap(response => {
      if (response.error) {
        const errorDetails = extractErrorDetails(response.error)
        return Effect.zipRight(
          Effect.logError("Accept terms function returned an error in its response.", {
            error: errorDetails,
          }),
          Effect.fail(new SupabaseError({ operation: "acceptTerms", cause: errorDetails })),
        )
      }

      // Check if the function returned an error in the response data
      if (response.data && !response.data.success && response.data.error) {
        return Effect.fail(
          new SupabaseError({
            operation: "acceptTerms",
            cause: { message: response.data.error, name: "EdgeFunctionError" },
          }),
        )
      }

      return Effect.succeed(Option.fromNullable(response.data as AcceptTermsResponse))
    }),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ cause: error, operation: "acceptTerms" }))
      return Effect.succeed(Option.none())
    }),
  )

/** Verify human status using Authena API */
export const verifyHuman = (): Effect.Effect<
  Option.Option<VerifyHumanResponse>,
  never,
  SupabaseClient
> =>
  pipe(
    SupabaseClient,
    Effect.flatMap((client) =>
      Effect.tryPromise({
        try: () =>
          client.functions.invoke("airdrop-verify-human", {
            body: {},
          }),
        catch: (error) =>
          new SupabaseError({
            operation: "verifyHuman",
            cause: extractErrorDetails(error as Error),
          }),
      })
    ),
    Effect.retry(retryForever),
    Effect.flatMap(response => {
      if (response.error) {
        const errorDetails = extractErrorDetails(response.error)
        return Effect.zipRight(
          Effect.logError("Verify human function returned an error in its response.", {
            error: errorDetails,
          }),
          Effect.fail(new SupabaseError({ operation: "verifyHuman", cause: errorDetails })),
        )
      }

      // Check if the function returned an error in the response data
      if (response.data && !response.data.success && response.data.error) {
        return Effect.fail(
          new SupabaseError({
            operation: "verifyHuman",
            cause: { message: response.data.error, name: "EdgeFunctionError" },
          }),
        )
      }

      return Effect.succeed(Option.fromNullable(response.data as VerifyHumanResponse))
    }),
    Effect.catchAll((error) => {
      errorStore.showError(new SupabaseError({ cause: error, operation: "verifyHuman" }))
      return Effect.succeed(Option.none())
    }),
  )
