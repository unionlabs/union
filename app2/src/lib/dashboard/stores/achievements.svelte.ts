import { runFork, runPromise } from "$lib/runtime"
import { Duration, Effect, Fiber, Option, pipe } from "effect"
import type { Entity } from "../client"
import { AchievementError } from "../errors"
import {
  type Achievement as QueryAchievement,
  getAvailableAchievements,
  getUserAchievements,
} from "../queries/index"
import { errorStore } from "../stores/errors.svelte"

export type Achievement = QueryAchievement & {
  category?: { title: string } | null
  subcategory?: { title: string } | null
  end_at?: string | null
  reward_achievements?: {
    rewards: {
      created_at: string
      cutoff: string | null
      default_handled: boolean
      default_requires_handling: boolean
      description: string | null
      id: number
      meta: any
      title: string | null
      type: number | null
    }
  }[]
}

export type UserAchievement = Entity<"user_achievements">

const POLL_INTERVAL = 5 * 60_000

export class AchievementsStore {
  /** Achievements the current user has earned */
  achieved = $state<Option.Option<Array<UserAchievement>>>(Option.none())
  /** All achievements that can be earned (filtered to exclude expired ones) */
  available = $state<Option.Option<Array<Achievement>>>(Option.none())

  /** Polling fiber */
  private pollFiber: Fiber.Fiber<never, Error> | null = null

  /** Achievements organized by chain (includes expired achievements for visual indication) */
  achievementByChain = $derived(
    Option.flatMap(this.available, (achievements) => {
      const achievementsMap = new Map(
        achievements.map((achievement) => [achievement.id, achievement]),
      )

      // Helper function to find the first achievement in a chain
      const findChainStart = (achievement: Achievement): Achievement => {
        const previous = achievements.find((a) => a.next === achievement.id)
        return previous ? findChainStart(previous) : achievement
      }

      return Option.some(
        achievements.reduce<Array<Array<Achievement>>>((chains, achievement) => {
          // Skip if achievement is already in any chain
          if (chains.some((chain) => chain.some((a) => a.id === achievement.id))) {
            return chains
          }

          // Find the actual start of this chain
          const chainStart = findChainStart(achievement)

          // Build chain from the start
          const chain: Array<Achievement> = []
          let current: Achievement | null = chainStart

          while (current) {
            chain.push(current)
            current = current.next !== null
              ? achievementsMap.get(current.next) ?? null
              : null
          }

          chains.push(chain)
          return chains
        }, []),
      )
    }),
  )

  constructor(private readonly userId: string) {
    this.loadUserAchievements(userId)
    this.loadAvailableAchievements()
    this.startPolling(userId)
  }

  /**
   * Loads user achievements for a specific user
   * @param userId - The ID of the user to load achievements for
   */
  private loadUserAchievements(userId: string) {
    runPromise(
      pipe(
        getUserAchievements(userId),
        Effect.tap((result) => {
          this.achieved = result
          return Effect.void
        }),
        Effect.catchAll((error) => {
          errorStore.showError(
            new AchievementError({
              cause: error,
              operation: "load",
              message: "Failed to load user achievements",
            }),
          )
          return Effect.succeed(Option.none())
        }),
      ),
    )
  }

  /**
   * Checks if an achievement has expired based on its end_at date
   * @param achievement - The achievement to check
   * @returns true if the achievement has expired, false otherwise
   */
  public isAchievementExpired(achievement: Achievement): boolean {
    // If no end_at date is set, the achievement never expires
    if (!achievement.end_at) {
      return false
    }

    // Parse the end_at date and check if it's in the past
    const endDate = new Date(achievement.end_at)
    const now = new Date()
    return endDate <= now
  }

  /**
   * Loads all available achievements (including expired ones for visual indication)
   */
  private loadAvailableAchievements() {
    runPromise(
      pipe(
        getAvailableAchievements(),
        Effect.tap((result) => {
          // Store all achievements without filtering expired ones
          this.available = result
          return Effect.void
        }),
        Effect.catchAll((error) => {
          errorStore.showError(
            new AchievementError({
              cause: error,
              operation: "loadAvailable",
              message: "Failed to load available achievements",
            }),
          )
          return Effect.succeed(Option.none())
        }),
      ),
    )
  }

  /**
   * Starts polling for achievement updates
   * @param userId - The user ID to poll achievements for
   * @private
   */
  private startPolling(userId: string) {
    this.stopPolling() // Make sure to stop any existing poll

    // Start polling fiber
    const self = this
    this.pollFiber = runFork(
      Effect.forever(
        pipe(
          getUserAchievements(userId),
          Effect.tap((result) => {
            self.achieved = result
            return Effect.void
          }),
          Effect.catchAll((error) => {
            errorStore.showError(
              new AchievementError({
                cause: error,
                operation: "load",
                message: "Failed to poll user achievements",
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
   * Stops polling for achievement updates
   * @private
   */
  private stopPolling() {
    if (this.pollFiber) {
      runPromise(Fiber.interrupt(this.pollFiber))
      this.pollFiber = null
    }
  }

  /**
   * Refreshes all achievements for the current user
   */
  refresh() {
    this.loadUserAchievements(this.userId)
    this.loadAvailableAchievements()
  }

  /**
   * Cleans up resources when the store is no longer needed
   */
  cleanup() {
    this.stopPolling()
    this.achieved = Option.none()
    this.available = Option.none()
  }
}
