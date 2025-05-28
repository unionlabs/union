import { runForkWithRuntime, runPromiseExitWithRuntime } from "$lib/utils/effect.svelte.js"
import type { PriceOracle } from "@unionlabs/sdk/PriceOracle"
import { Layer, ManagedRuntime, Match, pipe } from "effect"
import { isNotUndefined } from "effect/Predicate"
import type { GraphQL } from "./graphql/service"

const IS_VITEST = isNotUndefined(import.meta.vitest)

// TOOD: determine how to handle error channel due to dynamic imports in services
type AppLayer = Layer.Layer<GraphQL | PriceOracle, never, never>
export type AppContext = Layer.Layer.Success<AppLayer>

const make = async () => {
  const AppLayer = (await pipe(
    Match.value(IS_VITEST),
    Match.when(true, () => import("$lib/layers/test.js")),
    Match.when(false, () => import("$lib/layers/live.js")),
    Match.exhaustive,
  )).default as AppLayer satisfies AppLayer
  // XXX: ^ remove cast after handling layer construction errors

  const {
    runFork,
    runPromise,
    runPromiseExit,
    runSync,
    runSyncExit,
    runtime: _runtime,
  } = ManagedRuntime.make(AppLayer)

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

type Runtime = Awaited<ReturnType<typeof make>>

export let runFork$: Runtime["runFork$"]
export let runFork: Runtime["runFork"]
export let runPromise: Runtime["runPromise"]
export let runPromiseExit$: Runtime["runPromiseExit$"]
export let runPromiseExit: Runtime["runPromiseExit"]
export let runSync: Runtime["runSync"]
export let runSyncExit: Runtime["runSyncExit"]

export const __init = async () => {
  const runtime = await make()
  ;({
    runFork$,
    runFork,
    runPromise,
    runPromiseExit$,
    runPromiseExit,
    runSync,
    runSyncExit,
  } = runtime)
}
