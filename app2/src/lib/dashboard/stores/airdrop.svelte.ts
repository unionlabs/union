import { runFork, runPromise } from "$lib/runtime"
import { Duration, Effect, Fiber, Option, pipe } from "effect"
import { CACHE_VERSION } from "../config"
import { SupabaseError } from "../errors"
import { getUserAllocation, getUserClaim } from "../queries"
import type { UserAllocation, UserClaim } from "../queries/types"
import { clearLocalStorageCacheEntry } from "../services/cache"
import { errorStore } from "../stores/errors.svelte"

export class AirdropStore {
  allocation = $state<Option.Option<UserAllocation>>(Option.none())
  claim = $state<Option.Option<UserClaim>>(Option.none())

  isLoadingClaim = $state(false)

  generateError = $state<string | null>(null)
  claimError = $state<string | null>(null)

  private pollFiber: Fiber.RuntimeFiber<void, never> | null = null

  hasClaim = $derived(
    Option.match(this.claim, {
      onNone: () => false,
      onSome: () => true,
    }),
  )

  constructor(private userId: string) {
    this.loadData()
    this.startPolling()
  }

  private loadData() {
    if (!this.userId) {
      return
    }

    runPromise(
      pipe(
        Effect.sync(() => {
          this.isLoadingClaim = true
        }),
        Effect.flatMap(() =>
          Effect.all([
            getUserAllocation(this.userId),
            getUserClaim(this.userId),
          ])
        ),
        Effect.tap(([allocationResult, claimResult]) =>
          Effect.sync(() => {
            if (Option.isSome(allocationResult)) {
              this.allocation = allocationResult
            }
            if (Option.isSome(claimResult)) {
              this.claim = claimResult
            }
          })
        ),
        Effect.catchAll((error) =>
          pipe(
            Effect.sync(() =>
              errorStore.showError(
                new SupabaseError({
                  operation: "loadClaim",
                  cause: error as Error,
                }),
              )
            ),
            Effect.flatMap(() => Effect.succeed(Option.none())),
          )
        ),
        Effect.ensuring(Effect.sync(() => {
          this.isLoadingClaim = false
        })),
      ),
    )
  }

  /** Refresh claim data by clearing cache first, then loading */
  refresh() {
    if (!this.userId) {
      return
    }

    runPromise(
      pipe(
        clearLocalStorageCacheEntry("user_claim", `${CACHE_VERSION}:${this.userId}`),
        Effect.flatMap(() => Effect.sync(() => this.loadData())),
        Effect.catchAll(() => Effect.void),
      ),
    )
  }

  /** Start polling for data updates */
  private startPolling() {
    this.pollFiber = runFork(
      pipe(
        Effect.sleep(Duration.minutes(5)),
        Effect.flatMap(() => Effect.sync(() => this.loadData())),
        Effect.forever,
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
    this.claim = Option.none()
    this.isLoadingClaim = false
    this.generateError = null
    this.claimError = null
  }

  /** Cleanup method for store destruction */
  cleanup() {
    this.reset()
  }
}
