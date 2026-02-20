import { Runtime } from "@unionlabs/effect-svelte"
import type { PriceOracle } from "@unionlabs/sdk/PriceOracle"
import type { ZkgmWasm } from "@unionlabs/sdk/ZkgmWasm"
import { Layer, ManagedRuntime, Match, pipe } from "effect"
import { isNotUndefined } from "effect/Predicate"
import type { SupabaseClient } from "./dashboard/client"
import type { GasPriceMap } from "./gasprice"
import type { GraphQL } from "./graphql/service"

const IS_VITEST = isNotUndefined(import.meta.vitest)

// TODO: determine how to handle error channel due to dynamic imports in services
type AppLayer = Layer.Layer<
  SupabaseClient | GasPriceMap | GraphQL | PriceOracle | ZkgmWasm,
  never,
  never
>
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

  const runFork$ = Runtime.runForkWithRuntime(runtime)
  const runPromiseExit$ = Runtime.runPromiseExitWithRuntime(runtime)

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

/** @public */
export let runFork$: Runtime["runFork$"]
/** @public */
export let runFork: Runtime["runFork"]
/** @public */
export let runPromise: Runtime["runPromise"]
/** @public */
export let runPromiseExit$: Runtime["runPromiseExit$"]
/** @public */
export let runPromiseExit: Runtime["runPromiseExit"]
/** @public */
export let runSync: Runtime["runSync"]
/** @public */
export let runSyncExit: Runtime["runSyncExit"]

/** @public */
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
