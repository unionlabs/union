# Union Svelte + Effect Library

> \[!CAUTION\]
> This package is experimental and subject to breaking changes.

## Modules

- `Runtime`: Provides management of Effect runtime in Svelte components and reactive rune-like Effect execution.
- `Snippets`: Provides Svelte snippets to perform matching on common Effect ADTs (e.g. `Option`, `Either`, `Exit`, etc.)
- `SvelteConfigProvider`: Provides [`ConfigProvider`](https://effect-ts.github.io/effect/effect/ConfigProvider.ts.html#configprovider) for common SvelteKit environment patterns.

## Getting Started

In lieu of a known alterative, it is suggested to initialize the runtime as a module singleton and to trigger construction in a client hook.

### Runtime Definition

Define layers and re-export helper functions.

`src/lib/runtime.ts`

```ts
import { Runtime } from "@unionlabs/effect-svelte"
import { Layer, ManagedRuntime, Match, Predicate, pipe } from "effect"

/**
 * Ensure "vitest/importMeta" is defined in `tsconfig.json` "types".
 */
const IS_VITEST = Predicate.isNotUndefined(import.meta.vitest)

type AppLayer = Layer.Layer<never, never, never>
export type AppContext = Layer.Layer.Success<AppLayer>

const make = async () => {
  const AppLayer = (
    await pipe(
      Match.value(IS_VITEST),
      Match.when(true, () => import("$lib/layers/test.js")),
      Match.when(false, () => import("$lib/layers/live.js")),
      Match.exhaustive
    )
  ).default satisfies AppLayer

  const {
    runFork,
    runPromise,
    runPromiseExit,
    runSync,
    runSyncExit,
    runtime: _runtime
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
    runSyncExit
  } = runtime)
}
```

### Runtime Initialization

This client hook guarantees that the Effect runtime is initalized as early as possbile.

`src/hooks.client.ts`:

```ts
import type { ClientInit } from "@sveltejs/kit"

export const init: ClientInit = async () => {
  await import("$lib/runtime.js").then((_) => _.__init())
}
```

### Usage

See [test cases](https://github.com/unionlabs/union/blob/main/effect-svelte/lib/test/Runtime.test.ts) for inspiration.
