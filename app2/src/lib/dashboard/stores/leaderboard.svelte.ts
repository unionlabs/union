import { extractErrorDetails } from "@unionlabs/sdk/utils"
import { Duration, Effect, Fiber, Option, pipe } from "effect"
import type { Entity } from "../client"
import { LeaderboardError } from "../errors"
import { getLeaderboard } from "../queries/index"

export type UserExperience = Entity<"user_levels">

const POLL_INTERVAL = 5 * 60_000

export class LeaderboardStore {
  /** Current leaderboard data */
  leaderboard = $state<Option.Option<Array<UserExperience>>>(Option.none())

  /** Polling fiber */
  private pollFiber: Fiber.Fiber<never, Error> | null = null

  /**
   * Top 10 users by experience
   * @example
   * ```ts
   * dashboard.leaderboard.top10 // Get top 10 users
   * ```
   */
  top10 = $derived(
    Option.flatMap(this.leaderboard, (users) => Option.some(users.slice(0, 10))).pipe(
      Option.getOrElse(() => []),
    ),
  )

  /**
   * Total number of users on the leaderboard
   * @example
   * ```ts
   * dashboard.leaderboard.totalUsers // Get total number of users
   * ```
   */
  totalUsers = $derived(
    Option.flatMap(this.leaderboard, (users) => Option.some(users.length)).pipe(
      Option.getOrElse(() => 0),
    ),
  )

  constructor() {
    console.log("[leaderboard] Initializing LeaderboardStore")
    this.loadLeaderboard()
    this.startPolling()
  }

  /**
   * Loads leaderboard data
   * @private
   */
  private loadLeaderboard() {
    Effect.runPromise(
      pipe(
        getLeaderboard(),
        Effect.tap((result) => {
          console.log("[leaderboard] Leaderboard loaded:", {
            hasData: Option.isSome(result),
            dataLength: Option.isSome(result) ? result.value.length : 0,
          })
          this.leaderboard = result
          return Effect.void
        }),
        Effect.catchAll((error) =>
          Effect.fail(
            new LeaderboardError({
              cause: extractErrorDetails(error),
              operation: "load",
            }),
          )
        ),
      ),
    )
  }

  /**
   * Starts polling for leaderboard updates
   * @private
   */
  private startPolling() {
    this.stopPolling() // Make sure to stop any existing poll

    // Start polling fiber
    const self = this
    this.pollFiber = Effect.runFork(
      Effect.forever(
        pipe(
          getLeaderboard(),
          Effect.tap((result) => {
            console.log("[leaderboard] Polling update:", {
              hasData: Option.isSome(result),
              dataLength: Option.isSome(result) ? result.value.length : 0,
            })
            self.leaderboard = result
            return Effect.void
          }),
          Effect.catchAll((error) =>
            Effect.fail(
              new LeaderboardError({
                cause: extractErrorDetails(error),
                operation: "load",
              }),
            )
          ),
          Effect.delay(Duration.millis(POLL_INTERVAL)),
        ),
      ),
    )
  }

  /**
   * Stops polling for leaderboard updates
   * @private
   */
  private stopPolling() {
    if (this.pollFiber) {
      Effect.runPromise(Fiber.interrupt(this.pollFiber))
      this.pollFiber = null
    }
  }

  /**
   * Refreshes leaderboard data
   */
  refresh() {
    this.loadLeaderboard()
  }

  /**
   * Cleans up resources when the store is no longer needed
   */
  cleanup() {
    this.stopPolling()
    this.leaderboard = Option.none()
  }
}
