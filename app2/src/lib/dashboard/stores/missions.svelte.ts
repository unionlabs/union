import { Duration, Effect, Fiber, Option, pipe } from "effect"
import type { Entity } from "../client"
import { MissionError } from "../errors"
import { getAvailableMissions, getUserMissions } from "../queries/index"
import { errorStore } from "../stores/errors.svelte"

export type Mission = Entity<"missions">
export type UserMission = Entity<"user_missions">

const POLL_INTERVAL = 5 * 60_000

export class MissionsStore {
  /** All available missions in the system */
  available = $state<Option.Option<Array<Mission>>>(Option.none())

  /** User's mission progress data */
  progress = $state<Option.Option<Array<UserMission>>>(Option.none())

  /** Polling fiber */
  private pollFiber: Fiber.Fiber<never, Error> | null = null

  /**
   * Missions enhanced with user progress and timing information
   * @example
   * ```ts
   * dashboard.missions.enhanced // Get all missions with progress
   * ```
   */
  enhanced = $derived(
    Option.flatMap(this.available, (missions) =>
      Option.flatMap(this.progress, (userMissions) => {
        console.log("[mission] Computing enhanced missions:", {
          available: missions,
          progress: userMissions,
        })
        const now = new Date()
        console.log("[mission] Current time:", now.toISOString())
        return Option.some(
          missions.map((mission) => {
            const userMission = userMissions.find((um) => um.mission_id === mission.id)
            const progress = userMission?.progression ?? 0
            const threshold = userMission?.threshold ?? 0
            const isExpired = new Date(mission.end) <= now
            const isFuture = new Date(mission.start) > now
            const isCurrent = new Date(mission.start) <= now && new Date(mission.end) > now

            console.log("[mission] Processing mission:", {
              id: mission.id,
              title: mission.title,
              start: mission.start,
              end: mission.end,
              isExpired,
              isFuture,
              isCurrent,
              hasUserMission: !!userMission,
              progress,
              threshold,
            })

            return {
              ...mission,
              progress,
              threshold,
              completed: threshold > 0 && progress >= threshold,
              started: userMission !== undefined,
              percentComplete: threshold > 0
                ? Math.min((progress / threshold) * 100, 100)
                : 0,
              isExpired,
              isFuture,
              isCurrent,
              completed_at: userMission?.completed_at,
            }
          }),
        )
      })).pipe(
        Option.getOrElse(() => {
          console.log("[mission] No enhanced missions available")
          return []
        }),
      ),
  )

  /**
   * Missions that have been completed, sorted by completion date
   * @example
   * ```ts
   * dashboard.missions.completed // Get completed missions
   * ```
   */
  completed = $derived(
    this.enhanced
      .filter((m) => m.completed)
      .sort((a, b) => {
        const dateA = new Date(a.completed_at ?? 0)
        const dateB = new Date(b.completed_at ?? 0)
        return dateB.getTime() - dateA.getTime()
      }),
  )

  /**
   * Currently active missions in progress, sorted by completion percentage
   * @example
   * ```ts
   * dashboard.missions.active // Get active missions
   * ```
   */
  active = $derived(
    this.enhanced
      .filter((m) => {
        const isActive = !m.completed && m.isCurrent && m.started
        console.log("[mission] Checking active mission:", {
          id: m.id,
          title: m.title,
          completed: m.completed,
          isCurrent: m.isCurrent,
          started: m.started,
          isActive,
        })
        return isActive
      })
      .sort((a, b) => b.percentComplete - a.percentComplete),
  )

  /**
   * Upcoming missions sorted by start date
   * @example
   * ```ts
   * dashboard.missions.upcoming // Get upcoming missions
   * ```
   */
  upcoming = $derived(
    this.enhanced
      .filter((m) => {
        const isUpcoming = !m.completed && m.isFuture
        console.log("[mission] Checking upcoming mission:", {
          id: m.id,
          title: m.title,
          completed: m.completed,
          isFuture: m.isFuture,
          isUpcoming,
        })
        return isUpcoming
      })
      .sort((a, b) => new Date(a.start).getTime() - new Date(b.start).getTime()),
  )

  /**
   * Failed missions that expired before completion
   * @example
   * ```ts
   * dashboard.missions.failed // Get failed missions
   * ```
   */
  failed = $derived(
    this.active.filter((mission) => new Date(mission.end) <= new Date()),
  )

  /**
   * Missions that are past their end date and were not completed.
   * @example
   * ```ts
   * dashboard.missions.expiredUncompleted // Get expired and not completed missions
   * ```
   */
  expiredUncompleted = $derived(
    this.enhanced.filter(m => m.isExpired && !m.completed)
      .sort((a, b) => new Date(b.end).getTime() - new Date(a.end).getTime()), // Show most recently expired first
  )

  /**
   * Overall mission statistics and completion rates
   * @example
   * ```ts
   * dashboard.missions.stats // Get mission statistics
   * ```
   */
  stats = $derived.by(() => {
    // Count all missions regardless of status
    const totalCount = this.enhanced.length
    const completedCount = this.completed.length

    return {
      total: totalCount, // Total of all missions
      inProgress: this.active.length,
      completed: completedCount,
      upcoming: this.upcoming.length,
      completionRate: totalCount > 0
        ? (completedCount / totalCount) * 100
        : 0,
    }
  })

  constructor(private readonly userId: string) {
    console.log("[mission] Initializing MissionsStore for user:", userId)
    this.loadUserMissions(userId)
    this.loadAvailableMissions()
    this.startPolling(userId)
  }

  /**
   * Loads user missions data
   * @param userId - The ID of the user to load missions for
   * @private
   */
  private loadUserMissions(userId: string) {
    Effect.runPromise(
      pipe(
        getUserMissions(userId),
        Effect.tap((result) => {
          this.progress = result
          return Effect.void
        }),
        Effect.catchAll((error) => {
          errorStore.showError(
            new MissionError({
              cause: error,
              operation: "load",
              message: "Failed to load user missions",
            }),
          )
          return Effect.succeed(Option.none())
        }),
      ),
    )
  }

  /**
   * Loads all available missions
   * @private
   */
  private loadAvailableMissions() {
    Effect.runPromise(
      pipe(
        getAvailableMissions(),
        Effect.tap((result) => {
          this.available = result
          return Effect.void
        }),
        Effect.catchAll((error) => {
          errorStore.showError(
            new MissionError({
              cause: error,
              operation: "loadAvailable",
              message: "Failed to load available missions",
            }),
          )
          return Effect.succeed(Option.none())
        }),
      ),
    )
  }

  /**
   * Starts polling for mission updates
   * @param userId - The user ID to poll missions for
   * @private
   */
  private startPolling(userId: string) {
    this.stopPolling()

    const self = this
    this.pollFiber = Effect.runFork(
      Effect.forever(
        pipe(
          getUserMissions(userId),
          Effect.tap((result) => {
            self.progress = result
            return Effect.void
          }),
          Effect.catchAll((error) => {
            errorStore.showError(
              new MissionError({
                cause: error,
                operation: "load",
                message: "Failed to poll user missions",
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
   * Stops polling for mission updates
   * @private
   */
  private stopPolling() {
    if (this.pollFiber) {
      Effect.runPromise(Fiber.interrupt(this.pollFiber))
      this.pollFiber = null
    }
  }

  /**
   * Refreshes all mission data for the current user
   */
  refresh() {
    this.loadUserMissions(this.userId)
    this.loadAvailableMissions()
  }

  /**
   * Cleans up resources when the store is no longer needed
   */
  cleanup() {
    this.stopPolling()
    this.available = Option.none()
    this.progress = Option.none()
  }
}
