import { type Cause, type Effect, Either, Exit, type Fiber, FiberId, Option, Runtime } from "effect"
import { dual } from "effect/Function"
import type { Simplify } from "effect/Types"

/* eslint-disable @typescript-eslint/no-explicit-any, prefer-rest-params,  */
/**
 * @see https://github.com/Effect-TS/effect/blob/4a687e8dbe57702833d162a007a9f29863e514af/packages/effect/src/internal/runtime.ts#L29
 */
const makeDual = <Args extends Array<any>, Return>(
  f: (runtime: Runtime.Runtime<never>, effect: Effect.Effect<any, any>, ...args: Args) => Return,
): {
  <R>(runtime: Runtime.Runtime<R>): <A, E>(effect: Effect.Effect<A, E, R>, ...args: Args) => Return
  <R, A, E>(runtime: Runtime.Runtime<R>, effect: Effect.Effect<A, E, R>, ...args: Args): Return
} =>
  function(this: any) {
    // biome-ignore lint/style/noArguments: <explanation>
    if (arguments.length === 1) {
      // biome-ignore lint/style/noArguments: <explanation>
      const runtime = arguments[0]
      return (effect: any, ...args: Args) => f(runtime, effect, ...args)
    }
    // biome-ignore lint/style/noArguments: <explanation>
    return f.apply(this, arguments as any)
  } as any
/* eslint-enable @typescript-eslint/no-explicit-any, prefer-rest-params,  */

export type RunPromiseExitResult<A, E> = {
  readonly current: Option.Option<Exit.Exit<A, E>>
  readonly either: Option.Option<Either.Either<A, Cause.Cause<E>>>
  readonly interrupt: (reason?: any) => void
}
/**
 * {@link Effect.runPromiseExit} but like {@link $effect}.
 *
 * **TODO:**
 * - This ignores errors; either enforce no known errors or catch all {@link Cause<void>}?
 */
type RunPromiseExitOptions = {
  ignoreInterrupt?: boolean | undefined
}
export const runPromiseExitWithRuntime: {
  <R = never>(
    runtime: Runtime.Runtime<R>,
  ): <A, E, R>(
    effect: () => Effect.Effect<A, E, R>,
    options?: RunPromiseExitOptions | undefined,
  ) => Simplify<RunPromiseExitResult<A, E>>
  <A, E, R = never>(
    effect: () => Effect.Effect<A, E, R>,
    runtime: Runtime.Runtime<R>,
    options?: RunPromiseExitOptions | undefined,
  ): Simplify<RunPromiseExitResult<A, E>>
} = dual(
  3,
  <A, E, R>(
    effect: () => Effect.Effect<A, E, R>,
    runtime: Runtime.Runtime<R>,
    options?: RunPromiseExitOptions | undefined,
  ): Simplify<RunPromiseExitResult<A, E>> => {
    const runPromiseExit = Runtime.runPromiseExit(runtime)
    let state = $state<Option.Option<Exit.Exit<A, E>>>(Option.none())
    let controller = new AbortController()

    $effect(() => {
      controller = new AbortController()
      runPromiseExit(
        effect().pipe(
          Effect.allowInterrupt,
        ),
        { signal: controller.signal },
      ).then(Option.some)
        .then(x => {
          state = x
        })
      return () => controller.abort("teardown")
    })

    return {
      get current() {
        return state
      },
      get either() {
        return Option.map(
          state,
          Exit.match({
            onFailure: Either.left,
            onSuccess: Either.right,
          }),
        )
      },
      interrupt: (reason?: any) => controller.abort(reason),
    } as const
  },
)

export type RunForkResult<A, E> = {
  readonly fiber: Fiber.RuntimeFiber<A, E>
  readonly interrupt: () => void
}
/**
 * {@link Effect.runFork} with automatic {@link Fiber} cleanup.
 */
export const runForkWithRuntime: {
  <R>(
    runtime: Runtime.Runtime<R>,
  ): <A, E>(
    effect: Effect.Effect<A, E, R>,
    options?: Runtime.RunForkOptions | undefined,
  ) => Simplify<RunForkResult<A, E>>
  <R, A, E>(
    runtime: Runtime.Runtime<R>,
    effect: Effect.Effect<A, E, R>,
    options?: Runtime.RunForkOptions | undefined,
  ): Simplify<RunForkResult<A, E>>
} = makeDual(
  <R, A, E>(
    runtime: Runtime.Runtime<R>,
    self: Effect.Effect<A, E, R>,
    options?: Runtime.RunForkOptions,
  ): Simplify<RunForkResult<A, E>> => {
    const runFork = Runtime.runFork(runtime)
    let state = $state<Fiber.RuntimeFiber<A, E> | null>(null)

    $effect(() => {
      state = runFork(self, { immediate: true, ...options })
      return () => state?.unsafeInterruptAsFork(FiberId.none)
    })

    return {
      get fiber() {
        // biome-ignore lint/style/noNonNullAssertion: immediate execution
        return state!
      },
      interrupt: () => state?.unsafeInterruptAsFork(FiberId.none),
    } as const
  },
)
