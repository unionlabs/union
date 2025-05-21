import { runFork, runPromise } from "$lib/runtime"
import { Duration, Effect, Fiber, Option, pipe } from "effect"
import type { Entity } from "../client"
import { LeaderboardError } from "../errors"
import { getLeaderboard } from "../queries/index"
import { errorStore } from "../stores/errors.svelte"

export type UserExperience = Entity<"user_levels">

const POLL_INTERVAL = 5 * 60_000

export class LeaderboardStore {
  /** Current leaderboard data */
  leaderboard = $state<Option.Option<Array<UserExperience>>>(Option.none())

  /** Polling fiber */
  private pollFiber: Fiber.Fiber<never, Error> | null = null

  /**
   * Top 10 users by experience
   */
  top10 = $derived(
    Option.flatMap(this.leaderboard, (users) => Option.some(users.slice(0, 10))).pipe(
      Option.getOrElse(() => []),
    ),
  )

  /**
   * Total number of users on the leaderboard
   */
  totalUsers = $derived(
    Option.flatMap(this.leaderboard, (users) => Option.some(users.length)).pipe(
      Option.getOrElse(() => 0),
    ),
  )

  constructor() {
    this.loadLeaderboard()
    this.startPolling()
  }

  /**
   * Loads leaderboard data
   * @private
   */
  private loadLeaderboard() {
    runPromise(
      pipe(
        getLeaderboard(),
        Effect.tap((result) => {
          this.leaderboard = result
          return Effect.void
        }),
        Effect.catchAll((error) => {
          errorStore.showError(
            new LeaderboardError({
              cause: error,
              operation: "load",
              message: "Failed to load leaderboard data",
            }),
          )
          return Effect.succeed(Option.none())
        }),
      ),
    )
  }

  /**
   * Starts polling for leaderboard updates
   * @private
   */
  private startPolling() {
    this.stopPolling()

    const self = this
    this.pollFiber = runFork(
      Effect.forever(
        pipe(
          getLeaderboard(),
          Effect.tap((result) => {
            self.leaderboard = result
            return Effect.void
          }),
          Effect.catchAll((error) => {
            errorStore.showError(
              new LeaderboardError({
                cause: error,
                operation: "load",
                message: "Failed to poll leaderboard data",
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
   * Stops polling for leaderboard updates
   * @private
   */
  private stopPolling() {
    if (this.pollFiber) {
      runPromise(Fiber.interrupt(this.pollFiber))
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
