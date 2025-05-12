import { runForkWithRuntime, runPromiseExitWithRuntime } from "$lib/utils/effect.svelte.js"
import { Layer, ManagedRuntime, Match, pipe } from "effect"
import { isNotUndefined } from "effect/Predicate"

const IS_VITEST = isNotUndefined(import.meta.vitest)

type AppLayer = Layer.Layer<never, never, never>

const make = async () => {
  const layer = (await pipe(
    Match.value(IS_VITEST),
    Match.when(true, () => import("$lib/layers/test.js")),
    Match.when(false, () => import("$lib/layers/live.js")),
    Match.orElseAbsurd,
  )).default satisfies AppLayer

  const {
    runFork,
    runPromise,
    runPromiseExit,
    runSync,
    runSyncExit,
    runtime: _runtime,
  } = ManagedRuntime.make(layer)

  const runtime = await _runtime()

  const runFork$ = runForkWithRuntime(runtime)
  const runPromiseExit$ = runPromiseExitWithRuntime(runtime)

  return {
    runFork$,
    runFork,
    runPromise,
    runPromiseExit$,
    runPromiseExit,
    runSync,
    runSyncExit,
  } as const
}

export const {
  runFork$,
  runFork,
  runPromise,
  runPromiseExit$,
  runPromiseExit,
  runSync,
  runSyncExit,
} = await make()
