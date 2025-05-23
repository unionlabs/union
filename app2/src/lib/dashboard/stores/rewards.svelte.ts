import { runFork, runPromise } from "$lib/runtime"
import { Duration, Effect, Fiber, Option, pipe } from "effect"
import type { Entity } from "../client"
import { RewardError } from "../errors"
import { getAvailableRewards, getUserRewards } from "../queries/index"
import { errorStore } from "../stores/errors.svelte"

export type Reward = Entity<"rewards">
export type UserReward = Entity<"user_rewards_with_queue">

const POLL_INTERVAL = 5 * 60_000

export class RewardsStore {
  /** User's earned rewards */
  earned = $state<Option.Option<UserReward[]>>(Option.none())

  /** All available rewards in the system */
  availableRewards = $state<Option.Option<Reward[]>>(Option.none())

  /** Polling fiber */
  private pollFiber: Fiber.Fiber<never, Error> | null = null

  /**
   * Rewards enhanced with user progress and status information
   */
  enhanced = $derived(
    Option.flatMap(
      this.availableRewards,
      (rewards) =>
        Option.flatMap(this.earned, (userRewards) => {
          return Option.some(
            rewards.map((reward) => {
              const userReward = userRewards.find((ur) => ur.reward_id === reward.id)
              const isClaimed = userReward?.created_at != null
              const isQueued = userReward?.enqueued_at != null
              const isHandled = userReward?.handled ?? false

              return {
                ...reward,
                claimed: isClaimed,
                queued: isQueued,
                handled: isHandled,
                claimed_at: userReward?.created_at,
                queued_at: userReward?.enqueued_at,
                handled_at: userReward?.handled ? userReward?.created_at : null,
                status: isHandled
                  ? "handled"
                  : isClaimed
                  ? "claimed"
                  : isQueued
                  ? "queued"
                  : "available",
              }
            }),
          )
        }),
    ).pipe(
      Option.getOrElse(() => {
        return []
      }),
    ),
  )

  /**
   * Rewards that have been claimed by the user
   */
  claimed = $derived(
    this.enhanced
      .filter((r) => r.claimed)
      .sort((a, b) => {
        const dateA = new Date(a.claimed_at ?? 0)
        const dateB = new Date(b.claimed_at ?? 0)
        return dateB.getTime() - dateA.getTime()
      }),
  )

  /**
   * Rewards that are currently queued for processing
   */
  queued = $derived(
    this.enhanced
      .filter((r) => r.queued && !r.handled)
      .sort((a, b) => {
        const dateA = new Date(a.queued_at ?? 0)
        const dateB = new Date(b.queued_at ?? 0)
        return dateA.getTime() - dateB.getTime()
      }),
  )

  /**
   * Rewards that have been fully handled
   */
  handled = $derived(
    this.enhanced
      .filter((r) => r.handled)
      .sort((a, b) => {
        const dateA = new Date(a.handled_at ?? 0)
        const dateB = new Date(b.handled_at ?? 0)
        return dateB.getTime() - dateA.getTime()
      }),
  )

  /**
   * Rewards that are available to be claimed
   */
  available = $derived(
    this.enhanced
      .filter((r) => !r.claimed && !r.queued && !r.handled)
      .sort((a, b) => (b.type ?? 0) - (a.type ?? 0)),
  )

  /**
   * Overall reward statistics
   */
  stats = $derived({
    total: this.enhanced.length,
    claimed: this.claimed.length,
    queued: this.queued.length,
    handled: this.handled.length,
    available: this.available.length,
    claimRate: this.enhanced.length > 0
      ? (this.claimed.length / this.enhanced.length) * 100
      : 0,
  })

  constructor(private readonly userId: string) {
    this.loadUserRewards(userId)
    this.loadAvailableRewards()
    this.startPolling(userId)
  }

  /**
   * Loads user rewards data
   * @param userId - The ID of the user to load rewards for
   * @private
   */
  private loadUserRewards(userId: string) {
    runPromise(
      pipe(
        getUserRewards(userId),
        Effect.tap((result) => {
          this.earned = result
          return Effect.void
        }),
        Effect.catchAll((error) => {
          errorStore.showError(
            new RewardError({
              cause: error,
              operation: "load",
              message: "Failed to load user rewards",
            }),
          )
          return Effect.succeed(Option.none())
        }),
      ),
    )
  }

  /**
   * Loads all available rewards
   * @private
   */
  private loadAvailableRewards() {
    runPromise(
      pipe(
        getAvailableRewards(),
        Effect.tap((result) => {
          this.availableRewards = result
          return Effect.void
        }),
        Effect.catchAll((error) => {
          errorStore.showError(
            new RewardError({
              cause: error,
              operation: "loadAvailable",
              message: "Failed to load available rewards",
            }),
          )
          return Effect.succeed(Option.none())
        }),
      ),
    )
  }

  /**
   * Starts polling for reward updates
   * @param userId - The user ID to poll rewards for
   * @private
   */
  private startPolling(userId: string) {
    this.stopPolling() // Make sure to stop any existing poll

    // Start polling fiber
    const self = this
    this.pollFiber = runFork(
      Effect.forever(
        pipe(
          getUserRewards(userId),
          Effect.tap((result) => {
            self.earned = result
            return Effect.void
          }),
          Effect.catchAll((error) => {
            errorStore.showError(
              new RewardError({
                cause: error,
                operation: "load",
                message: "Failed to poll user rewards",
              }),
            )
            return Effect.succeed(Option.none())
          }),
          Effect.delay(Duration.millis(POLL_INTERVAL)),
        ),
      ),
    )
  }

  /**
   * Stops polling for reward updates
   * @private
   */
  private stopPolling() {
    if (this.pollFiber) {
      runPromise(Fiber.interrupt(this.pollFiber))
      this.pollFiber = null
    }
  }

  /**
   * Refreshes all reward data for the current user
   */
  refresh() {
    this.loadUserRewards(this.userId)
    this.loadAvailableRewards()
  }

  /**
   * Cleans up resources when the store is no longer needed
   */
  cleanup() {
    this.stopPolling()
    this.earned = Option.none()
    this.availableRewards = Option.none()
  }

  /**
   * Updates all reward data for the current user
   * @param force - If true, forces an immediate refresh even if polling is active
   */
  async updateRewards(force = false) {
    // If not forcing and we have a polling fiber, let the polling handle the update
    if (!force && this.pollFiber) {
      return
    }

    // Stop current polling if it exists
    if (this.pollFiber) {
      this.stopPolling()
    }

    // Update both rewards in parallel
    await Promise.all([
      runPromise(
        pipe(
          getUserRewards(this.userId),
          Effect.tap((result) => {
            this.earned = result
            return Effect.void
          }),
        ),
      ),
      runPromise(
        pipe(
          getAvailableRewards(),
          Effect.tap((result) => {
            this.availableRewards = result
            return Effect.void
          }),
        ),
      ),
    ])

    // Restart polling if it was active
    if (this.pollFiber) {
      this.startPolling(this.userId)
    }
  }
}
