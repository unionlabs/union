import { browser } from "$app/environment"
import { runSync } from "$lib/runtime.js"
import { type ClientInit } from "@sveltejs/kit"
import type { Handle } from "@sveltejs/kit"
import { Data, Effect, identity, Match, Option, pipe } from "effect"
import { isString } from "effect/Predicate"

class UncaughtError extends Data.TaggedError("UncaughtError")<{
  cause: Error | undefined
}> {}

class UnhandledRejection extends Data.TaggedError("UnhandledRejection")<{
  cause?: Error | undefined
}> {}

const cleanupStorage = () => {
  const MIN_QUOTA_IN_MB = 1
  const isStorageFull = (storage: Storage): boolean => {
    try {
      const testKey = "quota_check"
      const sizeInBytes = MIN_QUOTA_IN_MB * 1024 * 1024
      const value = "u".repeat(sizeInBytes)
      storage.setItem(testKey, value)
      storage.removeItem(testKey)
      return false
    } catch (e) {
      return (e instanceof DOMException
        && (e.code === 22 || e.name === "QuotaExceededError")
        && storage.length !== 0)
    }
  }

  if (isStorageFull(localStorage)) {
    console.info("Cleaning storage...")
    Object.keys(localStorage).forEach(key => {
      localStorage.removeItem("quota_check")
      if (key.startsWith("graphql")) {
        localStorage.removeItem(key)
      }
    })
  }
}

export const init: ClientInit = async () => {
  await import("$lib/runtime").then(x => x.__init())

  try {
    cleanupStorage()
  } catch (e) {
    console.error("Failed to utilize local storage.")
  }

  window.onerror = (event, source, lineno, colno, error) => {
    const message = Match.value(event).pipe(
      Match.when(isString, identity),
      Match.when(Match.instanceOfUnsafe(Event), (x) => x),
      Match.exhaustive,
    )
    Effect.logError(message, new UncaughtError({ cause: error })).pipe(
      Effect.annotateLogs({
        event,
        source,
        lineno,
        colno,
      }),
      runSync,
    )
    // prevent default
    return true
  }
  window.onunhandledrejection = (event) => {
    const message = Match.value(event.reason).pipe(
      Match.when(isString, identity<string>),
      Match.when(Match.instanceOfUnsafe(Error), error => error.message),
      Match.orElse(() => "no message"),
    )
    const cause = Match.value(event.reason).pipe(
      Match.when(Match.instanceOfUnsafe(Error), identity<Error>),
      Match.orElse(() => undefined),
    )
    Effect.logError(message, new UnhandledRejection({ cause })).pipe(
      runSync,
    )
    event.preventDefault()
  }
}

const PROTECTED_PATHS = ["/dashboard"]

export const handle: Handle = async ({ event, resolve }) => {
  const dashboard = await import("$lib/dashboard/stores/user.svelte").then(x => x.dashboard)
  if (browser && PROTECTED_PATHS.some(path => event.url.pathname.startsWith(path))) {
    return Effect.runPromise(
      pipe(
        Effect.succeed(dashboard.session),
        Effect.flatMap(session =>
          Option.isNone(session)
            ? Effect.succeed(
              new Response("Redirect", {
                status: 302,
                headers: { Location: "/" },
              }),
            )
            : Effect.succeed(resolve(event))
        ),
      ),
    )
  }

  return resolve(event)
}
