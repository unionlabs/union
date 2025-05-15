import {
  Cause,
  Context,
  Deferred,
  Effect,
  Exit,
  Fiber,
  FiberId,
  FiberStatus,
  Layer,
  ManagedRuntime,
  Option,
  pipe,
  Runtime,
  RuntimeFlags,
  Stream,
} from "effect"
import { flushSync } from "svelte"
import { assert, describe, expect, it } from "vitest"
import { runForkWithRuntime, runPromiseExitWithRuntime } from "./effect.svelte.js"

const runFork = runForkWithRuntime(Runtime.defaultRuntime)
const runPromiseExit = runPromiseExitWithRuntime(Runtime.defaultRuntime)

type AppliedReturn<F, A> = F extends (_: A) => infer R ? R : never

describe("Effect runes", () => {
  const anyRuntimeFiberId = FiberId.runtime(expect.any(Number), expect.any(Number))

  describe("runPromiseExit", () => {
    it("executes", async ({ onTestFinished }) => {
      const asyncEffect = Effect.promise(async () => 1)
      let result = $state<AppliedReturn<typeof runPromiseExit, typeof asyncEffect> | null>(null)
      const cleanup = $effect.root(() => {
        result = runPromiseExit(asyncEffect)
      })
      assert.strictEqual(result?.current, Option.none())
      await expect.poll(() => result?.current).toEqual(Option.some(Exit.succeed(1)))
      onTestFinished(cleanup)
    })
    it("interrupts on cleanup", async () => {
      const latch = Deferred.unsafeMake<void>(FiberId.none)
      const asyncEffect = pipe(
        Deferred.complete(latch, Effect.void),
        Effect.andThen(() => pipe(Effect.void, Effect.forever)),
      )
      let result = $state<AppliedReturn<typeof runPromiseExit, typeof asyncEffect> | null>(null)
      const cleanup = $effect.root(() => {
        result = runPromiseExit(asyncEffect)
        return () => result?.interrupt("some reason")
      })
      await Effect.runPromise(Deferred.await(latch))
      cleanup()
      await expect
        .poll(() => result?.current)
        .toStrictEqual(
          expect.objectContaining(
            pipe(Cause.interrupt(anyRuntimeFiberId), Exit.failCause, Option.some),
          ),
        )
    })
    it("is interruptible", async ({ onTestFinished }) => {
      const latch = Deferred.unsafeMake<void>(FiberId.none)
      const asyncEffect = pipe(
        Deferred.complete(latch, Effect.void),
        Effect.andThen(() => pipe(Effect.void, Effect.forever)),
      )
      let result = $state<AppliedReturn<typeof runPromiseExit, typeof asyncEffect> | null>(null)
      const cleanup = $effect.root(() => {
        result = runPromiseExit(asyncEffect)
      })
      await Effect.runPromise(Deferred.await(latch))
      result?.interrupt()
      await expect
        .poll(() => result?.current)
        .toStrictEqual(
          expect.objectContaining(
            pipe(Cause.interrupt(anyRuntimeFiberId), Exit.failCause, Option.some),
          ),
        )
      onTestFinished(cleanup)
    })
    it("accepts runtime", async ({ onTestFinished }) => {
      class A extends Context.Tag("MyService")<A, { readonly _: Effect.Effect<number> }>() {}
      const managedRuntime = ManagedRuntime.make(Layer.succeed(A, { _: Effect.succeed(1) }))
      const runtime = await managedRuntime.runtime()
      const runForkLive = runForkWithRuntime(runtime)
      let result = $state<number>(0)
      const cleanup = $effect.root(() => {
        runForkLive(
          pipe(
            Effect.andThen(A, ({ _ }) => _),
            Effect.flatMap(x =>
              Effect.sync(() => {
                result = x
              })
            ),
          ),
        )
      })
      expect(result).toStrictEqual(0)
      await expect.poll(() => result).toStrictEqual(1)
      onTestFinished(cleanup)
    })
  })

  describe("runFork", () => {
    it("provisions fiber reference", ({ onTestFinished }) => {
      let result = $state<ReturnType<typeof runFork> | null>(null)
      const cleanup = $effect.root(() => {
        result = runFork(Effect.void.pipe(Effect.forever))
      })
      flushSync()
      // biome-ignore lint/style/noNonNullAssertion: flushed
      expect(Fiber.isRuntimeFiber(result!.fiber)).toBe(true)
      onTestFinished(cleanup)
    })

    it("can mutate $state rune", async ({ onTestFinished }) => {
      let count = $state(0)
      const latch = Deferred.unsafeMake<void>(FiberId.none)
      const cleanup = $effect.root(() => {
        runFork(
          pipe(
            Stream.repeatValue(0),
            Stream.take(3),
            Stream.mapEffect(() =>
              Effect.sync(() => {
                count += 1
              })
            ),
            Stream.onDone(() => Deferred.succeed(latch, undefined)),
            Stream.runDrain,
          ),
        )
      })
      expect(count).toBe(0)
      await Effect.runPromise(Deferred.await(latch))
      expect(count).toBe(3)
      onTestFinished(cleanup)
    })

    it("cleans up fork fiber", async () => {
      let result = $state<ReturnType<typeof runFork> | null>(null)
      const cleanup = $effect.root(() => {
        result = runFork(Effect.void.pipe(Effect.forever))
      })
      flushSync()
      // biome-ignore lint/style/noNonNullAssertion: flushed
      const running = await Effect.runPromise(result!.fiber.status)
      expect(running).toStrictEqual(
        FiberStatus.running(
          RuntimeFlags.make(
            RuntimeFlags.CooperativeYielding,
            RuntimeFlags.Interruption,
            RuntimeFlags.RuntimeMetrics,
          ),
        ),
      )
      cleanup()
      await expect
        // biome-ignore lint/style/noNonNullAssertion: flushed
        .poll(async () => Effect.runPromise(result!.fiber.status))
        .toStrictEqual(FiberStatus.done)
    })
  })
})
