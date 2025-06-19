import { runFork, runPromise } from "$lib/runtime"
import { extractErrorDetails } from "@unionlabs/sdk/utils/index"
import { Duration, Effect, Fiber, Option, pipe } from "effect"
import { CACHE_VERSION } from "../config"
import { SupabaseError } from "../errors"
import {
  acceptTerms,
  claimReferralCode,
  createAirdropEntry,
  generateReferralCode,
  getClaimedReferralCodesByUserId,
  getReferralCodesByUserId,
  getUserAllocation,
  removeReferralCode,
  scanAllocation,
  SUPPORTED_CHAINS,
  updatePreStake,
  updateTwitterData,
  verifyAirdropWallet,
  verifyHuman,
  verifyStargazeWallet,
  WALLET_VERIFICATION_MESSAGE,
} from "../queries"
import { type CurrentIncentives, getCurrentIncentives } from "../queries/public"
import type { SupportedChainId, UserAllocation } from "../queries/types"
import { clearLocalStorageCacheEntry } from "../services/cache"
import { errorStore } from "../stores/errors.svelte"

export interface UserAllocationCalculated extends UserAllocation {
  tokens_available_total: number
  tokens_remaining_for_prestaking: number
  tokens_remaining_for_sharing: number
}

export interface AppReferralCode {
  id: string
  code: string
  percentage: number
  status: "pending" | "claimed"
  createdAt: Date
  claimedAt: Date | null
  claimedBy: string | null
}

export interface AppClaimedCode {
  id: string
  code: string
  percentage: number
  claimedAt: Date
  originalOwner: string
}

export type ReferralCode = any

export class AirdropStore {
  allocation = $state<Option.Option<UserAllocation>>(Option.none())
  referralCodes = $state<Option.Option<Array<ReferralCode>>>(Option.none())
  claimedCodes = $state<Option.Option<Array<ReferralCode>>>(Option.none())
  currentIncentives = $state<Option.Option<CurrentIncentives>>(Option.none())

  isLoadingAllocation = $state(false)
  isGeneratingCode = $state(false)
  isClaimingCode = $state(false)
  isStaking = $state(false)

  generateError = $state<string | null>(null)
  claimError = $state<string | null>(null)

  calculatedAllocation = $derived<Option.Option<UserAllocationCalculated>>(
    Option.map(this.allocation, (allocation) => {
      // Maximum tokens user can stake based on their allocation percentage
      // But capped at their actual token balance (can't stake more tokens than they have)
      const maxStakeableTokens = Math.min(
        allocation.total_tokens,
        Math.floor(allocation.total_tokens * (allocation.allocation_percentage || 1)),
      )

      return {
        ...allocation,
        tokens_available_total: maxStakeableTokens,
        tokens_remaining_for_prestaking: Math.max(
          0,
          maxStakeableTokens - allocation.tokens_prestaked,
        ),
        // Note: tokens_remaining_for_sharing is not really needed since sharing is in percentage units
        tokens_remaining_for_sharing: Math.max(0, maxStakeableTokens - allocation.tokens_prestaked),
      }
    }),
  )

  private pollFiber: Fiber.RuntimeFiber<void, never> | null = null

  appReferralCodes = $derived(
    Option.map(this.referralCodes, (codes) =>
      codes.map((code): AppReferralCode => ({
        id: code.id,
        code: code.code,
        percentage: code.percentage * 100,
        status: code.is_claimed ? "claimed" : "pending",
        createdAt: new Date(code.created_at || new Date()),
        claimedAt: code.claimed_at ? new Date(code.claimed_at) : null,
        claimedBy: code.claimed_by_user_id,
      }))),
  )

  appClaimedCodes = $derived(
    Option.map(this.claimedCodes, (codes) =>
      codes.map((code): AppClaimedCode => ({
        id: code.id,
        code: code.code,
        percentage: code.percentage * 100,
        claimedAt: new Date(code.claimed_at!),
        originalOwner: code.user_id,
      }))),
  )

  /** Calculate total percentage allocated from all codes (pending + claimed) */
  totalAllocatedPercentage = $derived(
    this.referralCodes.pipe(
      Option.map((codes) => codes.reduce((total, code) => total + (code.percentage * 100), 0)), // Convert from 0-1 to 1-100 format
      Option.getOrElse(() => 0),
    ),
  )

  /** Calculate available percentage to share based on allocation */
  availableToSharePercentage = $derived.by(() => {
    const alloc = Option.getOrElse(this.calculatedAllocation, () => null)
    if (!alloc || !alloc.allocation_percentage) {
      return 0
    }

    // Calculate how much of the allocation percentage has been used for staking
    // This is the percentage of total tokens that have been staked
    const usedAllocationPercentage = alloc.tokens_prestaked / alloc.total_tokens

    // Available to share = total allocation percentage minus what's been used for staking
    // Both values are in decimal form (0.4 = 40%, 1.1 = 110%)
    const remainingAllocationPercentage = Math.max(
      0,
      alloc.allocation_percentage - usedAllocationPercentage,
    )

    // Convert to percentage (multiply by 100) and round to avoid floating-point issues
    return Math.round(remainingAllocationPercentage * 100 * 10000) / 10000
  })

  /** Get total allocation in tokens */
  totalAllocationTokens = $derived(
    this.calculatedAllocation.pipe(
      Option.map((alloc) => alloc.total_tokens),
      Option.getOrElse(() => 0),
    ),
  )

  /** Check if user has EVM wallet in their allocation */
  hasEvmWallet = $derived(
    Option.match(this.allocation, {
      onNone: () => false,
      onSome: (alloc) => alloc.evm_wallet !== null,
    }),
  )

  /** Check if user has Stargaze wallet in their allocation */
  hasStargazeWallet = $derived(
    Option.match(this.allocation, {
      onNone: () => false,
      onSome: (alloc) => alloc.stargaze_address !== null,
    }),
  )

  /** Check if user has accepted terms of service */
  hasAcceptedTerms = $derived(
    Option.match(this.allocation, {
      onNone: () => false,
      onSome: (alloc) => alloc.tos_accepted === true,
    }),
  )

  /** Check if user is human verified */
  isHuman = $derived(
    Option.match(this.allocation, {
      onNone: () => false,
      onSome: (alloc) => alloc.is_human,
    }),
  )

  /** Check if user is eligible for airdrop (has tokens > 0) */
  isEligible = $derived(
    Option.match(this.allocation, {
      onNone: () => false,
      onSome: (alloc) => alloc.total_tokens > 0,
    }),
  )

  /** Get current incentives percentage */
  incentivesPercentage = $derived(
    Option.match(this.currentIncentives, {
      onNone: () => 0,
      onSome: (incentives) => incentives.incentives_percent_effective,
    }),
  )

  constructor(private userId: string) {
    this.loadAllocationData()
    this.startPolling()
  }

  /** Helper: Create a loading state effect */
  private withLoadingState<A, E, R>(
    loadingFlag: () => boolean,
    setLoading: (loading: boolean) => void,
    effect: Effect.Effect<A, E, R>,
  ): Effect.Effect<A, E, R> {
    return pipe(
      Effect.sync(() => setLoading(true)),
      Effect.flatMap(() => effect),
      Effect.ensuring(Effect.sync(() => setLoading(false))),
    )
  }

  /** Helper: Clear all allocation-related caches */
  private clearAllocationCaches(): Effect.Effect<void, never, never> {
    const cacheKeySuffix = `${CACHE_VERSION}:${this.userId}`
    return pipe(
      Effect.all([
        clearLocalStorageCacheEntry("user_allocation", cacheKeySuffix),
        clearLocalStorageCacheEntry("referral_codes", cacheKeySuffix),
        clearLocalStorageCacheEntry("claimed_referral_codes", cacheKeySuffix),
        clearLocalStorageCacheEntry("public", `${CACHE_VERSION}:current_incentives`),
      ]),
      Effect.map(() => void 0),
      Effect.catchAll(() => Effect.succeed(void 0)), // Don't fail if cache clearing fails
    )
  }

  /** Helper: Clear only allocation cache (for staking updates) */
  private clearAllocationCache(): Effect.Effect<void, never, never> {
    const cacheKeySuffix = `${CACHE_VERSION}:${this.userId}`
    return pipe(
      clearLocalStorageCacheEntry("user_allocation", cacheKeySuffix),
      Effect.catchAll(() => Effect.succeed(void 0)),
    )
  }

  /** Helper: Clear allocation and claimed codes caches (for referral code claiming) */
  private clearAllocationAndClaimedCodesCache(): Effect.Effect<void, never, never> {
    const cacheKeySuffix = `${CACHE_VERSION}:${this.userId}`
    return pipe(
      Effect.all([
        clearLocalStorageCacheEntry("user_allocation", cacheKeySuffix),
        clearLocalStorageCacheEntry("claimed_referral_codes", cacheKeySuffix),
      ]),
      Effect.map(() => void 0),
      Effect.catchAll(() => Effect.succeed(void 0)),
    )
  }

  /** Refresh data by clearing cache first, then loading (preserves existing store values during fetch) */
  private refreshAllocationData() {
    if (!this.userId) {
      return
    }

    runPromise(
      pipe(
        this.clearAllocationCaches(),
        Effect.flatMap(() => Effect.sync(() => this.loadAllocationData())),
      ),
    )
  }

  /** Public method to refresh allocation data */
  refresh() {
    this.refreshAllocationData()
  }

  /** Accept terms of service */
  async acceptTermsOfService(): Promise<boolean> {
    if (!this.userId) {
      return false
    }

    return runPromise(
      pipe(
        acceptTerms(true),
        Effect.map((result) => {
          if (Option.isSome(result) && result.value.success) {
            // Refresh allocation data to update hasAcceptedTerms
            this.refresh()
            return true
          }
          return false
        }),
        Effect.catchAll((error) => {
          const errorDetails = extractErrorDetails(error as Error)
          console.error("Error accepting terms:", errorDetails)
          return Effect.succeed(false)
        }),
      ),
    )
  }

  /** Create airdrop entry */
  async createEntry(): Promise<boolean> {
    if (!this.userId) {
      return false
    }

    return runPromise(
      pipe(
        createAirdropEntry(),
        Effect.map((result) => {
          if (Option.isSome(result) && result.value.success) {
            // Refresh allocation data to get the new entry
            this.refresh()
            return true
          }
          return false
        }),
        Effect.catchAll((error) => {
          const errorDetails = extractErrorDetails(error as Error)
          console.error("Error creating airdrop entry:", errorDetails)
          return Effect.succeed(false)
        }),
      ),
    )
  }

  /** Scan for allocation using user's ID and any stored identifiers */
  async scanForMyAllocation(): Promise<boolean> {
    if (!this.userId) {
      return false
    }

    // Get current allocation to check for stored identifiers
    const currentAllocation = Option.getOrNull(this.allocation)

    const identifiers: {
      twitter_id?: string
      stargaze_address?: string
    } = {}

    if (currentAllocation?.twitter_id) {
      identifiers.twitter_id = currentAllocation.twitter_id
    }

    if (currentAllocation?.stargaze_address) {
      identifiers.stargaze_address = currentAllocation.stargaze_address
    }

    return runPromise(
      pipe(
        scanAllocation(
          identifiers.twitter_id,
          identifiers.stargaze_address,
        ),
        Effect.map((result) => {
          if (Option.isSome(result) && result.value.success) {
            // Refresh allocation data to get the populated tokens
            this.refresh()
            return true
          }
          return false
        }),
        Effect.catchAll((error) => {
          const errorDetails = extractErrorDetails(error as Error)
          console.error("Error scanning for allocation:", errorDetails)
          return Effect.succeed(false)
        }),
      ),
    )
  }

  /** Update allocation with Twitter data */
  async updateTwitter(twitter_id: string, twitter_username?: string): Promise<boolean> {
    if (!this.userId) {
      return false
    }

    return runPromise(
      pipe(
        updateTwitterData(twitter_id, twitter_username),
        Effect.map((result) => {
          if (Option.isSome(result) && result.value.success) {
            // Refresh allocation data to get the updated Twitter info
            this.refresh()
            return true
          }
          return false
        }),
        Effect.catchAll((error) => {
          const errorDetails = extractErrorDetails(error as Error)
          console.error("Error updating Twitter data:", errorDetails)
          return Effect.succeed(false)
        }),
      ),
    )
  }

  /** Refresh only allocation data (for staking updates) */
  private refreshAllocationOnly() {
    if (!this.userId) {
      return
    }

    runPromise(
      pipe(
        this.clearAllocationCache(),
        Effect.flatMap(() =>
          getUserAllocation(this.userId).pipe(
            Effect.tap((result) =>
              Effect.sync(() => {
                if (Option.isSome(result)) {
                  this.allocation = result
                }
              })
            ),
          )
        ),
        Effect.catchAll(() => Effect.void),
      ),
    )
  }

  /** Refresh allocation and claimed codes (for referral code claiming) */
  private refreshAllocationAndClaimedCodes() {
    if (!this.userId) {
      return
    }

    runPromise(
      pipe(
        this.clearAllocationAndClaimedCodesCache(),
        Effect.flatMap(() =>
          Effect.all([
            getUserAllocation(this.userId),
            getClaimedReferralCodesByUserId(this.userId),
          ])
        ),
        Effect.tap(([allocationResult, claimedCodesResult]) =>
          Effect.sync(() => {
            if (Option.isSome(allocationResult)) {
              this.allocation = allocationResult
            }
            if (Option.isSome(claimedCodesResult)) {
              this.claimedCodes = claimedCodesResult
            }
          })
        ),
        Effect.catchAll(() => Effect.void),
      ),
    )
  }

  /** Helper: Handle edge function errors */
  private handleEdgeFunctionError(operation: string, setError?: (error: string) => void) {
    return (error: unknown) =>
      pipe(
        Effect.sync(() => {
          const errorDetails = extractErrorDetails(error as Error)
          const errorMessage = errorDetails?.message || `Failed to ${operation}`

          if (setError) {
            setError(errorMessage)
          }

          errorStore.showError(
            new SupabaseError({
              operation,
              cause: errorDetails,
            }),
          )
        }),
        Effect.flatMap(() => Effect.succeed(Option.none())),
      )
  }

  /** Start polling for data updates */
  private startPolling() {
    this.pollFiber = runFork(
      pipe(
        Effect.sleep(Duration.minutes(5)),
        Effect.flatMap(() => Effect.sync(() => this.loadAllocationData())),
        Effect.forever,
      ),
    )
  }

  /** Load allocation and referral data for the current user */
  private loadAllocationData() {
    if (!this.userId) {
      return
    }

    runPromise(
      pipe(
        Effect.sync(() => {
          this.isLoadingAllocation = true
        }),
        Effect.flatMap(() =>
          Effect.all([
            getUserAllocation(this.userId),
            getReferralCodesByUserId(this.userId),
            getClaimedReferralCodesByUserId(this.userId),
            getCurrentIncentives(),
          ])
        ),
        Effect.map((
          [allocationResult, referralCodesResult, claimedCodesResult, incentivesResult],
        ) => ({
          allocation: allocationResult as Option.Option<UserAllocation>,
          referralCodes: referralCodesResult as Option.Option<Array<ReferralCode>>,
          claimedCodes: claimedCodesResult as Option.Option<Array<ReferralCode>>,
          currentIncentives: incentivesResult as Option.Option<CurrentIncentives>,
        })),
        Effect.tap((data) =>
          Effect.sync(() => {
            // Only update if we have data - preserve existing data during loading
            if (Option.isSome(data.allocation)) {
              this.allocation = data.allocation
            }
            if (Option.isSome(data.referralCodes)) {
              this.referralCodes = data.referralCodes
            }
            if (Option.isSome(data.claimedCodes)) {
              this.claimedCodes = data.claimedCodes
            }
            if (Option.isSome(data.currentIncentives)) {
              this.currentIncentives = data.currentIncentives
            }
          })
        ),
        Effect.catchAll((error) =>
          pipe(
            Effect.sync(() =>
              errorStore.showError(
                new SupabaseError({
                  operation: "loadAirdropData",
                  cause: error as Error,
                }),
              )
            ),
            Effect.flatMap(() => Effect.succeed(Option.none())),
          )
        ),
        Effect.ensuring(Effect.sync(() => {
          this.isLoadingAllocation = false
        })),
      ),
    )
  }

  /** Generate a new referral code */
  generateCode(percentage: number): void {
    if (!this.userId || this.isGeneratingCode) {
      return
    }

    runPromise(
      pipe(
        Effect.sync(() => {
          this.generateError = null
        }),
        Effect.flatMap(() =>
          this.withLoadingState(
            () => this.isGeneratingCode,
            (loading) => {
              this.isGeneratingCode = loading
            },
            generateReferralCode(this.userId, percentage),
          )
        ),
        Effect.flatMap((result) =>
          Option.isSome(result) && result.value.success
            ? Effect.sync(() => this.refreshAllocationData())
            : Effect.void
        ),
        Effect.catchAll(this.handleEdgeFunctionError("generate referral code", (error) => {
          this.generateError = error
        })),
      ),
    )
  }

  /** Claim a referral code */
  claimCode(code: string): Promise<boolean> {
    if (!this.userId || this.isClaimingCode) {
      return Promise.resolve(false)
    }

    return runPromise(
      pipe(
        Effect.sync(() => {
          this.claimError = null
        }),
        Effect.flatMap(() =>
          this.withLoadingState(
            () => this.isClaimingCode,
            (loading) => {
              this.isClaimingCode = loading
            },
            claimReferralCode(this.userId, code),
          )
        ),
        Effect.flatMap((result) =>
          Option.isSome(result) && result.value.success
            ? Effect.sync(() => {
              this.refreshAllocationAndClaimedCodes()
              return true
            })
            : Effect.sync(() => {
              this.claimError = "Failed to claim referral code"
              return false
            })
        ),
        Effect.catchAll((error) =>
          pipe(
            this.handleEdgeFunctionError("claim referral code", (err) => {
              this.claimError = err
            })(error),
            Effect.map(() => false),
          )
        ),
      ),
    )
  }

  /** Remove a referral code (only pending codes) */
  removeCode(codeId: string): void {
    if (!this.userId) {
      return
    }

    runPromise(
      pipe(
        removeReferralCode(codeId),
        Effect.flatMap((result) =>
          Option.isSome(result) && result.value.success
            ? Effect.sync(() => this.refreshAllocationData())
            : Effect.void
        ),
        Effect.catchAll(this.handleEdgeFunctionError("remove referral code")),
      ),
    )
  }

  /** Update staking amount via edge function */
  updateStaking(additionalAmount: number): Promise<boolean> {
    if (!this.userId || this.isStaking || additionalAmount <= 0) {
      return Promise.resolve(false)
    }

    return runPromise(
      pipe(
        this.withLoadingState(
          () => this.isStaking,
          (loading) => {
            this.isStaking = loading
          },
          updatePreStake(additionalAmount),
        ),
        Effect.flatMap((result) =>
          Option.isSome(result) && result.value.success
            ? Effect.sync(() => {
              this.refreshAllocationOnly()
              return true
            })
            : Effect.sync(() => false)
        ),
        Effect.catchAll((error) =>
          pipe(
            this.handleEdgeFunctionError("update pre-stake")(error),
            Effect.map(() => false),
          )
        ),
      ),
    )
  }

  /** Get the required message for wallet verification */
  getWalletVerificationMessage(): string {
    return WALLET_VERIFICATION_MESSAGE
  }

  /** Get supported chains */
  getSupportedChains() {
    return SUPPORTED_CHAINS
  }

  /** Verify airdrop wallet signature with the correct message format */
  verifyAirdropWalletSignature(
    walletAddress: string,
    signature: string,
    chainId: SupportedChainId,
  ): Promise<boolean> {
    if (!this.userId) {
      return Promise.resolve(false)
    }

    return runPromise(
      pipe(
        verifyAirdropWallet(walletAddress, WALLET_VERIFICATION_MESSAGE, signature, chainId),
        Effect.map((result) => {
          if (Option.isSome(result) && result.value.success) {
            this.refreshAllocationData()
            return true
          }
          return false
        }),
        Effect.catchAll((error) =>
          pipe(
            this.handleEdgeFunctionError("verify airdrop wallet", () => {})(error),
            Effect.map(() => false),
          )
        ),
      ),
    )
  }

  /** Verify Stargaze wallet and return success status */
  async verifyStargazeWalletWithSignature(
    address: string,
    message: string,
    signature: string,
  ): Promise<boolean> {
    if (!this.userId) {
      return Promise.resolve(false)
    }

    return runPromise(
      pipe(
        verifyStargazeWallet(address, message, signature, address),
        Effect.map((result) => {
          if (Option.isSome(result) && result.value.success) {
            // Refresh allocation data to get the updated wallet
            this.refreshAllocationData()
            return true
          }
          return false
        }),
        Effect.catchAll(() => Effect.succeed(false)),
      ),
    )
  }

  /** Verify human status using Authena API */
  verifyHumanWithAuthena(): Promise<boolean> {
    if (!this.userId) {
      return Promise.resolve(false)
    }

    return runPromise(
      pipe(
        verifyHuman(),
        Effect.map((result) => {
          if (Option.isSome(result) && result.value.success) {
            this.refreshAllocationData()
            return true
          }
          return false
        }),
        Effect.catchAll((error) =>
          pipe(
            this.handleEdgeFunctionError("verify human with Authena", () => {})(error),
            Effect.map(() => false),
          )
        ),
      ),
    )
  }

  /** Stop polling */
  stopPolling() {
    if (this.pollFiber) {
      runFork(Fiber.interrupt(this.pollFiber))
      this.pollFiber = null
    }
  }

  /** Clear error messages */
  clearErrors() {
    this.generateError = null
    this.claimError = null
  }

  /** Reset store state */
  reset() {
    this.stopPolling()
    this.allocation = Option.none()
    this.referralCodes = Option.none()
    this.claimedCodes = Option.none()
    this.currentIncentives = Option.none()
    this.isLoadingAllocation = false
    this.isGeneratingCode = false
    this.isClaimingCode = false
    this.isStaking = false
    this.generateError = null
    this.claimError = null
  }

  /** Cleanup method for store destruction */
  cleanup() {
    this.reset()
  }
}
