import { runSync } from "$lib/runtime.js"
import { type ClientInit } from "@sveltejs/kit"
import { Data, Effect, identity, Match } from "effect"
import { isString } from "effect/Predicate"

class UncaughtError extends Data.TaggedError("UncaughtError")<{
  cause: Error | undefined
}> {}

class UnhandledRejection extends Data.TaggedError("UnhandledRejection")<{
  cause?: Error | undefined
}> {}

export const init: ClientInit = async () => {
  await import("$lib/runtime").then(x => x.__init())
  await import("$lib/logging/datadog.js").then(x => x.__init())

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
