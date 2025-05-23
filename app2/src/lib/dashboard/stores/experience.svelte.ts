import { runFork, runPromise } from "$lib/runtime"
import { Duration, Effect, Fiber, Option, pipe } from "effect"
import type { Entity } from "../client"
import { ExperienceError } from "../errors"
import { getAvailableLevels, getUserExperience } from "../queries/index"
import { errorStore } from "../stores/errors.svelte"

export type Level = Entity<"levels">
export type UserExperience = Entity<"user_levels">

const POLL_INTERVAL = 10000 // 10 seconds

export class ExperienceStore {
  /** User's current experience and level data */
  current = $state<Option.Option<UserExperience>>(Option.none())

  /** Available levels in the system */
  levels = $state<Option.Option<Level[]>>(Option.none())

  /** Current user title */
  title = $state("")

  /** Polling fiber */
  private pollFiber: Fiber.Fiber<never, Error> | null = null

  /**
   * Current level progress as a percentage
   */
  progress = $derived(
    Option.flatMap(this.current, (exp) =>
      exp.current_xp != null && exp.xp_required != null
        ? Option.some((exp.current_xp / (exp.current_xp + exp.xp_required)) * 100)
        : Option.none()),
  )

  /**
   * Current level title based on total XP
   */
  level = $derived(
    Option.flatMap(this.current, (exp) =>
      Option.flatMap(this.levels, (levels) => {
        if (levels.length === 0) {
          return Option.none()
        }

        const sortedLevels = [...levels].sort((a, b) =>
          (b.experience_required ?? 0) - (a.experience_required ?? 0)
        )
        const level = sortedLevels.find(l => (l.experience_required ?? 0) <= (exp.total_xp ?? 0))
        return Option.fromNullable(level?.title)
      })),
  )

  /**
   * Next level information if available
   */
  next = $derived(
    Option.flatMap(this.current, (exp) =>
      Option.flatMap(this.levels, (levels) => {
        if (levels.length === 0) {
          return Option.none()
        }
        const next = levels.find(l => (l.experience_required ?? 0) > (exp.total_xp ?? 0))
        return Option.fromNullable(next)
      })),
  )

  /**
   * Maximum available level number
   */
  max = $derived(
    Option.flatMap(this.levels, (levels) => Option.some(levels.length)).pipe(
      Option.getOrElse(() => 0),
    ),
  )

  constructor(private readonly userId: string) {
    this.loadUserExperience(userId)
    this.loadAvailableLevels()
    this.startPolling(userId)
  }

  /**
   * Loads user experience data
   * @param userId - The ID of the user to load experience for
   * @private
   */
  private loadUserExperience(userId: string) {
    runPromise(
      pipe(
        getUserExperience(userId),
        Effect.tap((result) => {
          this.current = result
          return Effect.void
        }),
        Effect.catchAll((error) => {
          errorStore.showError(
            new ExperienceError({
              cause: error,
              operation: "load",
              message: "Failed to load user experience data",
            }),
          )
          return Effect.succeed(Option.none())
        }),
      ),
    )
  }

  /**
   * Loads all available levels
   * @private
   */
  private loadAvailableLevels() {
    runPromise(
      pipe(
        getAvailableLevels(),
        Effect.tap((result) => {
          this.levels = result
          return Effect.void
        }),
        Effect.catchAll((error) => {
          errorStore.showError(
            new ExperienceError({
              cause: error,
              operation: "loadLevels",
              message: "Failed to load available levels",
            }),
          )
          return Effect.succeed(Option.none())
        }),
      ),
    )
  }

  /**
   * Starts polling for experience updates
   * @param userId - The user ID to poll experience for
   * @private
   */
  private startPolling(userId: string) {
    this.stopPolling()

    const self = this
    this.pollFiber = runFork(
      Effect.forever(
        pipe(
          getUserExperience(userId),
          Effect.tap((result) => {
            self.current = result
            return Effect.void
          }),
          Effect.catchAll((error) => {
            errorStore.showError(
              new ExperienceError({
                cause: error,
                operation: "load",
                message: "Failed to poll user experience data",
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
   * Stops polling for experience updates
   * @private
   */
  private stopPolling() {
    if (this.pollFiber) {
      runPromise(Fiber.interrupt(this.pollFiber))
      this.pollFiber = null
    }
  }

  /**
   * Refreshes all experience data for the current user
   */
  refresh() {
    this.loadUserExperience(this.userId)
    this.loadAvailableLevels()
  }

  /**
   * Cleans up resources when the store is no longer needed
   */
  cleanup() {
    this.stopPolling()
    this.current = Option.none()
    this.levels = Option.none()
    this.title = ""
  }
}
