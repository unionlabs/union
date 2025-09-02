import { runFork, runPromise } from "$lib/runtime"
import { extractErrorDetails } from "@unionlabs/sdk/utils/index"
import { Duration, Effect, Fiber, Option, pipe } from "effect"
import { CACHE_VERSION } from "../config"
import { SupabaseError } from "../errors"
import { getUserClaim } from "../queries"
import type { UserClaim } from "../queries/types"
import { clearLocalStorageCacheEntry } from "../services/cache"
import { errorStore } from "../stores/errors.svelte"

export class AirdropStore {
  claim = $state<Option.Option<UserClaim>>(Option.none())

  isLoadingClaim = $state(false)

  generateError = $state<string | null>(null)
  claimError = $state<string | null>(null)

  private pollFiber: Fiber.RuntimeFiber<void, never> | null = null

  /** Check if user has a claim */
  hasClaim = $derived(
    Option.match(this.claim, {
      onNone: () => false,
      onSome: () => true,
    }),
  )

  constructor(private userId: string) {
    this.loadClaim()
    this.startPolling()
  }

  private loadClaim() {
    if (!this.userId) {
      return
    }

    runPromise(
      pipe(
        Effect.sync(() => {
          this.isLoadingClaim = true
        }),
        Effect.flatMap(() => getUserClaim(this.userId)),
        Effect.tap((claimResult) =>
          Effect.sync(() => {
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

  /** Refresh claim data by clearing cache first, then loading */
  refresh() {
    if (!this.userId) {
      return
    }

    runPromise(
      pipe(
        clearLocalStorageCacheEntry("user_claim", `${CACHE_VERSION}:${this.userId}`),
        Effect.flatMap(() => Effect.sync(() => this.loadClaim())),
        Effect.catchAll(() => Effect.void),
      ),
    )
  }

  /** Start polling for data updates */
  private startPolling() {
    this.pollFiber = runFork(
      pipe(
        Effect.sleep(Duration.minutes(5)),
        Effect.flatMap(() => Effect.sync(() => this.loadClaim())),
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
